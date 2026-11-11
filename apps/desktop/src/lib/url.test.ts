import { describe, expect, it } from "vitest";
import { hostOf, isAcceptableServiceUrl, isLoopbackUrl } from "./url";

describe("service addresses", () => {
  it("names the host that data is sent to", () => {
    expect(hostOf("https://openrouter.ai/api/v1")).toBe("openrouter.ai");
    expect(hostOf(" http://localhost:1234/v1 ")).toBe("localhost");
    expect(hostOf("not a url")).toBeNull();
  });

  it("recognizes addresses on this computer", () => {
    expect(isLoopbackUrl("http://localhost:1234/v1")).toBe(true);
    expect(isLoopbackUrl("http://127.0.0.1:8080")).toBe(true);
    expect(isLoopbackUrl("https://openrouter.ai/api/v1")).toBe(false);
    expect(isLoopbackUrl("http://localhost.example.com")).toBe(false);
  });

  it("accepts https anywhere and http only on this computer", () => {
    expect(isAcceptableServiceUrl("https://openrouter.ai/api/v1")).toBe(true);
    expect(isAcceptableServiceUrl("http://localhost:1234/v1")).toBe(true);
    expect(isAcceptableServiceUrl("http://openrouter.ai/api/v1")).toBe(false);
    expect(isAcceptableServiceUrl("ftp://openrouter.ai")).toBe(false);
    expect(isAcceptableServiceUrl("")).toBe(false);
  });
});
