/**
 * Unit tests for the multi-level category taxonomy: app-kind filtering,
 * node lookup, and breadcrumb path resolution.
 */
import { describe, expect, it } from "vitest";
import {
  categoriesForAppKind,
  categoryPathTo,
  findCategoryNode,
} from "../src/service/app-categories.ts";

describe("categoriesForAppKind", () => {
  it("returns the full taxonomy for an unspecified app kind", () => {
    const tree = categoriesForAppKind(undefined);
    expect(tree.length).toBeGreaterThan(3);
  });

  it("keeps subtrees whose leaves apply to the app kind", () => {
    // All nodes carry no appKind constraint, so every kind keeps the tree.
    const tree = categoriesForAppKind("IOS_APP");
    expect(tree.length).toBe(categoriesForAppKind(undefined).length);
  });
});

describe("findCategoryNode", () => {
  it("resolves a leaf id anywhere in the tree", () => {
    const node = findCategoryNode("dev-tools-cicd");
    expect(node?.id).toBe("dev-tools-cicd");
  });

  it("returns undefined for unknown ids", () => {
    expect(findCategoryNode("not-a-category")).toBeUndefined();
  });
});

describe("categoryPathTo", () => {
  it("builds the breadcrumb chain from the root to a leaf", () => {
    const path = categoryPathTo("entertainment-games-puzzle");
    expect(path.map((node) => node.id)).toEqual([
      "entertainment",
      "entertainment-games",
      "entertainment-games-puzzle",
    ]);
  });

  it("returns an empty chain for unknown ids", () => {
    expect(categoryPathTo("nope")).toEqual([]);
  });
});
