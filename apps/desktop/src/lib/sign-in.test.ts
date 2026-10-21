import { expect, test } from "vitest";

import { hostname, trusted } from "@/lib/sign-in";

const claude = ["claude.com", "claude.ai"];

test("trusts https links on a known domain or its subdomains", () => {
  expect(
    trusted("https://claude.com/cai/oauth/authorize?state=abc", claude),
  ).toBe(true);
  expect(trusted("https://auth.openai.com/codex/device", ["openai.com"])).toBe(
    true,
  );
});

test("rejects other hosts, imitations and other schemes", () => {
  expect(trusted("https://evil.example/claude.com", claude)).toBe(false);
  expect(trusted("https://claude.com.evil.example/", claude)).toBe(false);
  expect(trusted("https://notclaude.com/", claude)).toBe(false);
  expect(trusted("https://claude.com@evil.example/", claude)).toBe(false);
  expect(trusted("http://claude.com/", claude)).toBe(false);
  expect(trusted("file:///C:/Windows/System32/calc.exe", claude)).toBe(false);
  expect(trusted("not a url", claude)).toBe(false);
});

test("trusts only the GitHub Enterprise host the user signs in to", () => {
  for (const input of [
    "acme.ghe.com",
    "https://ACME.ghe.com/",
    " acme.ghe.com ",
  ]) {
    expect(hostname(input)).toBe("acme.ghe.com");
  }
  expect(hostname("github.example.org")).toBe("github.example.org");

  const acme = [hostname("acme.ghe.com")];
  expect(trusted("https://acme.ghe.com/login/device", acme)).toBe(true);
  expect(trusted("https://other.ghe.com/login/device", acme)).toBe(false);
  expect(trusted("https://github.com/login/device", acme)).toBe(false);
  expect(
    trusted("https://github.example.org/login/device", [
      hostname("https://github.example.org"),
    ]),
  ).toBe(true);
});
