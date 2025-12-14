import { test, expect, Page, Browser } from "@playwright/test";
import { GenericContainer } from "testcontainers";

// test.beforeEach(async () => {
//   const container = await GenericContainer.fromDockerfile(
//     "..",
//     "Dockerfile"
//   ).build();
//   const instance = await container.start();
// });

async function tabForward(page: Page, times: number) {
  for (let i = 0; i < times; i++) {
    await page.keyboard.press("Tab");
  }
}

async function readClipboard(page: Page) {
  return await page.evaluate(async () => {
    const clipboard = await navigator.clipboard.readText();
    console.log("CLIPBOARD", clipboard);
    return clipboard;
  });
}

async function setUpFriend(browser: Browser): Promise<{
  page: Page;
  [Symbol.asyncDispose](): Promise<void>;
}> {
  const friendContext = await browser.newContext();
  const friendPage = await friendContext.newPage();

  return { page: friendPage, [Symbol.asyncDispose]: friendContext.close };
}

test("A user can start a chat with another user", async ({
  page,
  context,
  browser,
}) => {
  const friendSetup = setUpFriend(browser);
  await page.goto("http://localhost:5173/invite");

  // Page ready
  const copyButton = page.getByText("Copy");
  // await page.locator("button:has-text('Copy')").waitFor({ state: "attached" });
  await copyButton.waitFor({ state: "visible" });

  // Go to copy button
  // await tabForward(page, 3);
  // const copyButton = page.locator("*:focus");
  // const copyButton = page.getByText("Copy");
  // expect(copyButton).toHaveText("Copy");
  // await page.keyboard.press("Enter");
  await copyButton.click();

  // await context.grantPermissions(["clipboard-read"]);
  const invite = await readClipboard(page);
  // Starts with "http://localhost:5173/join/"
  expect(invite).toMatch(/^http:\/\/localhost:5173\/join\//);



  await using friend = await friendSetup;
  await friend.page.goto(invite);
  const acceptButton = friend.page.getByText("Accept");
  await acceptButton.waitFor({ state: "visible" });

  const nameInputLabel = friend.page.getByText("Your name");
  await nameInputLabel.click();

  await friend.page.keyboard.type("test friend");

  // await friend.page.keyboard.press("Enter");
  await acceptButton.click();

  // Wait for navigation
  const chatUrlPattern = /^http:\/\/localhost:5173\/chat\//;
  await friend.page.waitForURL(chatUrlPattern);
  await friend.page.waitForSelector("h1:has-text('Chat')");

  await page.waitForURL(chatUrlPattern);



});
