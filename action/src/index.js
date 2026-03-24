import { appendFile, readFile } from "node:fs/promises";
import { extname } from "node:path";
import { fileURLToPath } from "node:url";

import {
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
  pretty_print_header,
  pretty_print_header_with_config_auto,
  validate_header,
  validate_header_with_config_auto,
} from "../../lib/rs_lib.internal.js";
import process from "node:process";

const COMMENT_MARKER = "<!-- conventional-prs-validation -->";
const CONFIG_PATHS = [
  ".github/semantic.yml",
  ".github/semantic.yaml",
  ".github/semantic.json",
  ".github/semantic.jsonc",
  ".github/semantic.toml",
];

let wasmReady;

export const readActionInput = (name, env = process.env) => {
  const normalized = name.replace(/ /gu, "_").toUpperCase();
  const key = `INPUT_${normalized}`;

  return env[key];
};

export const parseBooleanInput = (value, defaultValue = false) => {
  if (value === undefined || value === null || value.trim().length === 0) {
    return defaultValue;
  }

  const normalized = value.trim().toLowerCase();
  if (["1", "true", "yes", "on"].includes(normalized)) {
    return true;
  }
  if (["0", "false", "no", "off"].includes(normalized)) {
    return false;
  }

  return defaultValue;
};

export const inferConfigFormat = (path) => {
  switch (extname(path).toLowerCase()) {
    case ".yml":
      return "yml";
    case ".yaml":
      return "yaml";
    case ".json":
      return "json";
    case ".jsonc":
      return "jsonc";
    case ".toml":
      return "toml";
    default:
      return undefined;
  }
};

export const readPullRequest = (eventPayload) => {
  const pullRequest = eventPayload?.pull_request;
  const number = pullRequest?.number;
  const title = pullRequest?.title;

  if (!Number.isInteger(number)) {
    throw new Error("Invalid pull request number in event payload");
  }
  if (typeof title !== "string" || title.length === 0) {
    throw new Error("Missing pull request title in event payload");
  }

  return { number, title };
};

export const buildFailureComment = (report) => {
  return [
    COMMENT_MARKER,
    "## PR Title Validation Failed",
    "",
    "```",
    report,
    "```",
    "",
    "Your PR title must follow Conventional Commits.",
  ].join("\n");
};

const encodePath = (path) => path.split("/").map(encodeURIComponent).join("/");

const appendMultilineOutput = async (name, value) => {
  const outputFile = process.env.GITHUB_OUTPUT;
  if (!outputFile) {
    return;
  }

  await appendFile(
    outputFile,
    `${name}<<__CONVENTIONAL_PRS__\n${value}\n__CONVENTIONAL_PRS__\n`,
  );
};

const writeOutputs = async ({ valid, configPath, report }) => {
  await appendMultilineOutput("valid", valid ? "true" : "false");
  await appendMultilineOutput("config-path", configPath);
  await appendMultilineOutput("report", report);
};

const appendSummary = async (content) => {
  const summaryFile = process.env.GITHUB_STEP_SUMMARY;
  if (!summaryFile) {
    return;
  }

  await appendFile(summaryFile, `${content}\n`);
};

const githubRequest = (url, options = {}) => {
  const headers = new Headers(options.headers ?? {});
  if (options.token) {
    headers.set("Authorization", `Bearer ${options.token}`);
  }

  return fetch(url, {
    method: options.method ?? "GET",
    headers,
    body: options.body,
  });
};

const listValidationComments = async (
  { repository, pullRequestNumber, token },
) => {
  if (!token) {
    return [];
  }

  const url =
    `https://api.github.com/repos/${repository}/issues/${pullRequestNumber}/comments?per_page=100`;
  const response = await githubRequest(url, {
    token,
    headers: {
      Accept: "application/vnd.github+json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to list PR comments: ${response.status} ${response.statusText}`,
    );
  }

  const comments = await response.json();
  if (!Array.isArray(comments)) {
    throw new Error("Unexpected comment payload from GitHub API");
  }

  return comments.filter((comment) =>
    typeof comment?.body === "string" && comment.body.includes(COMMENT_MARKER)
  );
};

const upsertFailureComment = async (
  { repository, pullRequestNumber, token, report },
) => {
  if (!token) {
    return;
  }

  const body = buildFailureComment(report);
  const comments = await listValidationComments({
    repository,
    pullRequestNumber,
    token,
  });
  const existing = comments[0];

  if (existing) {
    const updateUrl =
      `https://api.github.com/repos/${repository}/issues/comments/${existing.id}`;
    const updateResponse = await githubRequest(updateUrl, {
      method: "PATCH",
      token,
      headers: {
        Accept: "application/vnd.github+json",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ body }),
    });

    if (!updateResponse.ok) {
      throw new Error(
        `Failed to update PR comment: ${updateResponse.status} ${updateResponse.statusText}`,
      );
    }

    return;
  }

  const createUrl =
    `https://api.github.com/repos/${repository}/issues/${pullRequestNumber}/comments`;
  const createResponse = await githubRequest(createUrl, {
    method: "POST",
    token,
    headers: {
      Accept: "application/vnd.github+json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ body }),
  });

  if (!createResponse.ok) {
    throw new Error(
      `Failed to create PR comment: ${createResponse.status} ${createResponse.statusText}`,
    );
  }
};

const deleteValidationComments = async (
  { repository, pullRequestNumber, token },
) => {
  if (!token) {
    return;
  }

  const comments = await listValidationComments({
    repository,
    pullRequestNumber,
    token,
  });
  for (const comment of comments) {
    const deleteUrl =
      `https://api.github.com/repos/${repository}/issues/comments/${comment.id}`;
    const response = await githubRequest(deleteUrl, {
      method: "DELETE",
      token,
      headers: {
        Accept: "application/vnd.github+json",
      },
    });

    if (!response.ok) {
      throw new Error(
        `Failed to delete PR comment: ${response.status} ${response.statusText}`,
      );
    }
  }
};

const loadRepoConfig = async ({ repository, token }) => {
  for (const path of CONFIG_PATHS) {
    const url = `https://api.github.com/repos/${repository}/contents/${
      encodePath(path)
    }`;
    const response = await githubRequest(url, {
      token,
      headers: {
        Accept: "application/vnd.github.raw+json",
      },
    });

    if (response.status === 404) {
      continue;
    }
    if (!response.ok) {
      throw new Error(
        `Failed to fetch ${path}: ${response.status} ${response.statusText}`,
      );
    }

    const content = await response.text();
    return {
      path,
      format: inferConfigFormat(path),
      content,
    };
  }

  return null;
};

const wasmUrls = [
  new URL("./rs_lib.wasm", import.meta.url),
  new URL("../../lib/rs_lib.wasm", import.meta.url),
];

const loadWasmBytes = async () => {
  for (const url of wasmUrls) {
    try {
      return await readFile(url);
    } catch (error) {
      if (error && typeof error === "object" && "code" in error) {
        const code = error.code;
        if (code === "ENOENT" || code === "ENOTDIR") {
          continue;
        }
      }
      throw error;
    }
  }

  throw new Error("Unable to locate bundled rs_lib.wasm");
};

const initWasm = async () => {
  const bytes = await loadWasmBytes();
  const { instance } = await WebAssembly.instantiate(bytes, {
    "./rs_lib.internal.js": {
      __wbindgen_init_externref_table,
    },
  });

  const wasmExports = instance.exports;
  __wbg_set_wasm(wasmExports);

  if (typeof wasmExports.__wbindgen_start === "function") {
    wasmExports.__wbindgen_start();
  } else {
    __wbindgen_init_externref_table();
  }
};

const ensureWasm = async () => {
  if (!wasmReady) {
    wasmReady = initWasm();
  }
  await wasmReady;
};

const parseValidationResult = (raw) => JSON.parse(raw);

const validateTitle = async ({ title, configFile }) => {
  await ensureWasm();

  const validation = configFile === null
    ? parseValidationResult(validate_header(title))
    : parseValidationResult(
      validate_header_with_config_auto(
        title,
        configFile.content,
        configFile.format,
      ),
    );

  if (validation.ok) {
    return {
      valid: true,
      report: "",
    };
  }

  const report = configFile === null
    ? pretty_print_header(title)
    : pretty_print_header_with_config_auto(
      title,
      configFile.content,
      configFile.format,
    );

  return {
    valid: false,
    report,
  };
};

export const run = async () => {
  const eventName = process.env.GITHUB_EVENT_NAME;
  if (eventName !== "pull_request" && eventName !== "pull_request_target") {
    console.log(
      "This action only runs on pull_request or pull_request_target events",
    );
    await writeOutputs({ valid: true, configPath: "", report: "" });
    return;
  }

  if (
    !parseBooleanInput(
      readActionInput("validate-pr-title") ??
        process.env.INPUT_VALIDATE_PR_TITLE,
      true,
    )
  ) {
    console.log("PR title validation is disabled");
    await writeOutputs({ valid: true, configPath: "", report: "" });
    return;
  }

  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) {
    throw new Error("GITHUB_EVENT_PATH is required");
  }

  const repository = process.env.GITHUB_REPOSITORY;
  if (!repository) {
    throw new Error("GITHUB_REPOSITORY is required");
  }

  const token = readActionInput("github-token") ?? process.env.GITHUB_TOKEN ??
    "";
  const eventPayload = JSON.parse(await readFile(eventPath, "utf8"));
  const pullRequest = readPullRequest(eventPayload);
  const configFile = await loadRepoConfig({ repository, token });

  if (configFile) {
    console.log(`Using config: ${configFile.path}`);
  } else {
    console.log("No repository config found, using defaults");
  }

  console.log(`Validating PR #${pullRequest.number}: ${pullRequest.title}`);

  const validation = await validateTitle({
    title: pullRequest.title,
    configFile,
  });

  await writeOutputs({
    valid: validation.valid,
    configPath: configFile?.path ?? "",
    report: validation.report,
  });

  if (validation.valid) {
    console.log("PR title is valid");
    await deleteValidationComments({
      repository,
      pullRequestNumber: pullRequest.number,
      token,
    });
    return;
  }

  const report = validation.report.trimEnd();
  console.error(report);
  await appendSummary(["```", report, "```"].join("\n"));
  await upsertFailureComment({
    repository,
    pullRequestNumber: pullRequest.number,
    token,
    report,
  });
  process.exitCode = 1;
};

const isMainModule = process.argv[1] === fileURLToPath(import.meta.url);

if (isMainModule) {
  run().catch(async (error) => {
    const message = error instanceof Error ? error.message : String(error);
    console.error(message);
    await writeOutputs({ valid: false, configPath: "", report: message });
    process.exitCode = 1;
  });
}
