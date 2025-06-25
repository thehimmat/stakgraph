import { SimpleEvaluator } from "./simple-evaluator.js";
import { TestResult } from "./types.js";

export async function evaluate(
  browser_url: string,
  test_url: string,
  prompt: string
): Promise<TestResult> {
  console.log("🚀 Starting simple evaluation...");
  console.log(`📝 Prompt: ${prompt}`);
  console.log(`🌐 Test URL: ${test_url}`);
  console.log(`🔧 Browser URL: ${browser_url}`);

  // Create evaluator with browser_url (empty string will be ignored due to if(browserUrl) check)
  const evaluator = new SimpleEvaluator(browser_url);

  try {
    const result = await evaluator.runTest(prompt, test_url);

    console.log("\n" + "=".repeat(50));
    console.log("📊 TEST RESULTS");
    console.log("=".repeat(50));
    console.log(`Status: ${result.status}`);
    console.log(`Description: ${result.description}`);

    if (result.failedCriteria) {
      console.log("\nFailed Criteria:");
      result.failedCriteria.forEach((criterion) => {
        console.log(`❌ ${criterion}`);
      });
    }

    console.log("\n" + "=".repeat(50));
    return result;
  } catch (error) {
    console.error("❌ Test failed:", error);
    throw error;
  }
}
