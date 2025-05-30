// src/sage/src/utils/config.ts
import * as fs from "fs";
import * as path from "path";

export interface Config {
  github: {
    owner: string;
    repo: string;
    app_id: string;
    private_key: string; // This will be the path to the private key file or the key itself
    installation_id: string;
  };
  codeSpaceURL: string;
  "2b_base_url": string;
  secret: string;
  stakwork_api_key: string;
  workflow_id: string;
  data_dir?: string;
  dry_run?: boolean;
  webhook_url?: string;
}

export function loadConfig(configPath: string = "sage_config.json"): Config {
  try {
    const configFilePath = path.resolve(process.cwd(), configPath);
    const configData = fs.readFileSync(configFilePath, "utf8");
    const config = JSON.parse(configData) as Config;

    // If private_key is a file path, read the file content
    if (
      config.github.private_key &&
      config.github.private_key.endsWith(".pem")
    ) {
      const keyPath = path.resolve(process.cwd(), config.github.private_key);
      config.github.private_key = fs.readFileSync(keyPath, "utf8");
    }

    return config;
  } catch (error) {
    // console.error(`Error loading config from ${configPath}:`, error);
    throw new Error(`Failed to load configuration: ${error}`);
  }
}
