import * as pr from "../mod.ts"

const configPath = Deno.env.get("BENCH_CONFIG_PATH") ?? ".github/semantic.yml"
const input = Deno.env.get("BENCH_INPUT") ?? "feat(api): add recovery"

pr.parse(pr.parseConfig(await Deno.readTextFile(configPath)), input)
