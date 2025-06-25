import { Stagehand } from "@browserbasehq/stagehand";
import { getProvider } from "./providers.js";

let STAGEHAND: Stagehand | null = null;

export async function getOrCreateStagehand() {
  if (STAGEHAND) {
    return STAGEHAND;
  }
  let provider = getProvider();
  console.log("initializing stagehand!", provider.model);
  const sh = new Stagehand({
    env: "LOCAL",
    domSettleTimeoutMs: 30000,
    localBrowserLaunchOptions: {
      headless: true,
      viewport: { width: 1024, height: 768 },
    },
    enableCaching: true,
    modelName: provider.model,
    modelClientOptions: {
      apiKey: process.env[provider.api_key_env_var_name],
    },
  });
  await sh.init();
  STAGEHAND = sh;
  return sh;
}

export function sanitize(bodyText: string) {
  const content = bodyText
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line &&
        !(
          (line.includes("{") && line.includes("}")) ||
          line.includes("@keyframes") ||
          line.match(/^\.[a-zA-Z0-9_-]+\s*{/) ||
          line.match(/^[a-zA-Z-]+:[a-zA-Z0-9%\s\(\)\.,-]+;$/)
        )
    )
    .map((line) =>
      line.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) =>
        String.fromCharCode(parseInt(hex, 16))
      )
    );
  return content;
}
