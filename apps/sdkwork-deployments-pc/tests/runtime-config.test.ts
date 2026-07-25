import { parseDeploymentsRuntimeConfig, resolveDeploymentsLocale } from "@sdkwork/deployments-pc-core";
import { describe, expect, it } from "vitest";

const locales = { defaultLocale: "zh-CN", fallbackLocale: "en-US", supportedLocales: ["zh-CN", "en-US"], activeLocales: ["zh-CN", "en-US"] };

describe("deployments runtime config", () => {
  it("accepts complete development config", () => expect(parseDeploymentsRuntimeConfig({ ...locales, environment: "development", deploymentProfile: "cloud", appApiBaseUrl: "http://127.0.0.1:3900", backendApiBaseUrl: "http://127.0.0.1:3900", driveAppApiBaseUrl: "http://127.0.0.1:3800", appbaseAppApiBaseUrl: "http://127.0.0.1:8080" }).environment).toBe("development"));
  it("rejects production loopback", () => expect(() => parseDeploymentsRuntimeConfig({ ...locales, environment: "production", deploymentProfile: "cloud", appApiBaseUrl: "http://127.0.0.1:3900", backendApiBaseUrl: "https://deploy.sdkwork.com", driveAppApiBaseUrl: "https://drive.sdkwork.com", appbaseAppApiBaseUrl: "https://iam.sdkwork.com" })).toThrow(/loopback/));
  it("resolves supported browser locale with explicit fallback", () => { const config = parseDeploymentsRuntimeConfig({ ...locales, environment: "development", deploymentProfile: "cloud", appApiBaseUrl: "http://127.0.0.1:3900", backendApiBaseUrl: "http://127.0.0.1:3900", driveAppApiBaseUrl: "http://127.0.0.1:3800", appbaseAppApiBaseUrl: "http://127.0.0.1:8080" }); expect(resolveDeploymentsLocale(config, ["zh-Hans-CN"])).toBe("zh-CN"); });
});
