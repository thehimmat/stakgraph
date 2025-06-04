import { Octokit } from "@octokit/rest";
import { BaseAdapter } from "./adapter.js";
import { Message } from "../types/index.js";
import * as fs from "fs";
import * as path from "path";
import { extractCodespaceUrl } from "../utils/markdown.js";

export class GitHubIssueAdapter extends BaseAdapter {
  private octokit: Octokit;
  private owner: string;
  private repo: string;
  private processedIssues: Map<number, string> = new Map(); // issue_number -> webhook_url
  private dataDir: string;
  private persistFilePath: string;

  constructor(
    githubToken: string,
    owner: string,
    repo: string,
    dataDir: string = "./data"
  ) {
    super();
    this.octokit = new Octokit({ auth: githubToken });
    this.owner = owner;
    this.repo = repo;
    this.dataDir = dataDir;
    this.persistFilePath = path.join(
      this.dataDir,
      `github-${owner}-${repo}-processed-issues.json`
    );
  }

  async initialize(): Promise<void> {
    console.log(`Initializing GitHub adapter for ${this.owner}/${this.repo}`);

    // Ensure data directory exists
    await this.ensureDataDirExists();

    // Load previously processed issues
    await this.loadProcessedIssues();

    // Start polling for new issues
    setInterval(async () => {
      try {
        await this.checkForNewIssues();
      } catch (error) {
        console.error("Error checking for new GitHub issues:", error);
      }
    }, 10000); // Check every 10 seconds
  }

  async sendResponse(chatId: string, message: Message): Promise<void> {
    // Extract issue number from chatId
    const issueNumber = parseInt(chatId.replace("github-issue-", ""), 10);

    if (isNaN(issueNumber)) {
      throw new Error(`Invalid GitHub issue chat ID: ${chatId}`);
    }

    console.log(`Sending message to GitHub issue #${issueNumber}`);
    await this.octokit.issues.createComment({
      owner: this.owner,
      repo: this.repo,
      issue_number: issueNumber,
      body: message.content,
    });
  }

  // Override the base storeWebhook method to handle GitHub specific storage
  override storeWebhook(chatId: string, webhook: string): void {
    // Extract issue number from chatId
    const issueNumber = parseInt(chatId.replace("github-issue-", ""), 10);

    if (isNaN(issueNumber)) {
      console.error(`Invalid GitHub issue chat ID for webhook storage: ${chatId}`);
      return;
    }

    // Store in both maps for compatibility
    super.storeWebhook(chatId, webhook); // Store in base class map
    this.storeWebhookForIssue(issueNumber, webhook); // Store in issue-specific map
  }

  // Override the base getWebhook method to handle GitHub specific retrieval
  override getWebhook(chatId: string): string | undefined {
    // Extract issue number from chatId
    const issueNumber = parseInt(chatId.replace("github-issue-", ""), 10);

    if (isNaN(issueNumber)) {
      console.error(`Invalid GitHub issue chat ID for webhook retrieval: ${chatId}`);
      return undefined;
    }

    // Try to get from issue-specific map first, fall back to base class map
    return this.getWebhookForIssue(issueNumber) || super.getWebhook(chatId);
  }

  // Legacy method maintained for compatibility
  public storeWebhookForIssue(issueNumber: number, webhook: string): void {
    this.processedIssues.set(issueNumber, webhook);
    // Save immediately when webhook is stored
    this.saveProcessedIssues().catch((error) => {
      console.error(
        "Error saving processed issues after webhook storage:",
        error
      );
    });
  }

  // Legacy method maintained for compatibility
  public getWebhookForIssue(issueNumber: number): string | undefined {
    return this.processedIssues.get(issueNumber);
  }

  private async ensureDataDirExists(): Promise<void> {
    if (!fs.existsSync(this.dataDir)) {
      await fs.promises.mkdir(this.dataDir, { recursive: true });
      console.log(`Created data directory: ${this.dataDir}`);
    }
  }

  private async loadProcessedIssues(): Promise<void> {
    try {
      if (fs.existsSync(this.persistFilePath)) {
        const data = await fs.promises.readFile(this.persistFilePath, "utf-8");
        const parsed = JSON.parse(data);

        // Handle both old format (array) and new format (object)
        if (Array.isArray(parsed)) {
          // Old format: convert array to Map with empty webhook strings
          this.processedIssues = new Map(parsed.map((id) => [id, ""]));
          console.log(
            `Loaded ${this.processedIssues.size} previously processed issues (old format)`
          );
        } else {
          // New format: object with issue_number -> webhook mapping
          this.processedIssues = new Map(
            Object.entries(parsed).map(([key, value]) => [
              parseInt(key),
              value as string,
            ])
          );
          console.log(
            `Loaded ${this.processedIssues.size} previously processed issues with webhooks`
          );
        }
      } else {
        console.log("No previously processed issues found");
      }
    } catch (error) {
      console.error("Error loading processed issues:", error);
      // Continue with empty map if loading fails
    }
  }

  private async saveProcessedIssues(): Promise<void> {
    try {
      // Convert Map to object for JSON storage
      const issuesObject = Object.fromEntries(this.processedIssues);
      await fs.promises.writeFile(
        this.persistFilePath,
        JSON.stringify(issuesObject, null, 2),
        "utf-8"
      );
      console.log(`Saved ${this.processedIssues.size} processed issues`);
    } catch (error) {
      console.error("Error saving processed issues:", error);
    }
  }

  async checkForNewIssues(): Promise<void> {
    // console.log("Checking for new GitHub issues...");
    const issues = await this.octokit.issues.listForRepo({
      owner: this.owner,
      repo: this.repo,
      state: "open",
      sort: "created",
      direction: "desc",
      per_page: 10,
    });

    let newIssuesProcessed = false;

    for (const issue of issues.data) {
      // Skip if already processed
      if (this.processedIssues.has(issue.number)) {
        continue;
      }
      if (!issue.body) {
        continue;
      }

      // Check if @stakwork or @stakgraph is mentioned
      if (
        issue.body.includes("@stakwork") ||
        issue.body.includes("@stakgraph")
      ) {
        const chatId = `github-issue-${issue.number}`;

        // Extract codespace URL from the issue body
        const extractedCodespaceUrl = extractCodespaceUrl(issue.body);

        const message: Message = {
          role: "user",
          content: issue.body,
          // Add extracted codespace URL as metadata
          ...(extractedCodespaceUrl && { codespaceUrl: extractedCodespaceUrl }),
        };

        console.log(
          `Processing GitHub issue #${issue.number} with @stakwork mention`
        );

        if (extractedCodespaceUrl) {
          console.log(`Extracted codespace URL: ${extractedCodespaceUrl}`);
        }

        this.processedIssues.set(issue.number, ""); // Store with empty webhook initially
        newIssuesProcessed = true;

        try {
          await this.messageCallback(chatId, message);
        } catch (error) {
          console.error(
            `Error processing GitHub issue #${issue.number}:`,
            error
          );
        }
      }
    }

    // Save processed issues if any new ones were added
    if (newIssuesProcessed) {
      await this.saveProcessedIssues();
    }
  }
}