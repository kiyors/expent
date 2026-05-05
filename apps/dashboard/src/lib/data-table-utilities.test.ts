import { describe, it, expect } from "vitest";
import {
  sortData,
  getRowIdentifier,
  createDataTableRowKeys,
  getDataTableMobileDescriptionId,
  parseNumericLike,
} from "./data-table-utilities";

describe("parseNumericLike", () => {
  it("should parse basic numbers", () => {
    expect(parseNumericLike("123")).toBe(123);
    expect(parseNumericLike("-123.45")).toBe(-123.45);
    expect(parseNumericLike("0")).toBe(0);
  });

  it("should handle whitespace", () => {
    expect(parseNumericLike("  123  ")).toBe(123);
    expect(parseNumericLike("1 234")).toBe(1234);
  });

  it("should handle accounting negatives", () => {
    expect(parseNumericLike("(1234)")).toBe(-1234);
    expect(parseNumericLike("(12.34)")).toBe(-12.34);
  });

  it("should strip currency and percent symbols", () => {
    expect(parseNumericLike("$1,234.56")).toBe(1234.56);
    expect(parseNumericLike("€1.234,56")).toBe(1234.56);
    expect(parseNumericLike("50%")).toBe(50);
    expect(parseNumericLike("¥100")).toBe(100);
  });

  it("should handle thousands and decimal separators", () => {
    // Standard US/UK
    expect(parseNumericLike("1,234,567.89")).toBe(1234567.89);
    // European style
    expect(parseNumericLike("1.234.567,89")).toBe(1234567.89);
    // Only comma
    expect(parseNumericLike("1234,56")).toBe(1234.56);
    expect(parseNumericLike("1,234")).toBe(1234);
    // Only dot
    expect(parseNumericLike("1234.56")).toBe(1234.56);
    expect(parseNumericLike("1.234")).toBe(1234);
  });

  it("should handle compact notation", () => {
    expect(parseNumericLike("500K")).toBe(500000);
    expect(parseNumericLike("1.5M")).toBe(1500000);
    expect(parseNumericLike("2.8T")).toBe(2800000000000);
    expect(parseNumericLike("1G")).toBe(1000000000);
  });

  it("should handle byte suffixes", () => {
    expect(parseNumericLike("1.5KB")).toBe(1.5 * 1024);
    expect(parseNumericLike("2GB")).toBe(2 * 1024 ** 3);
    expect(parseNumericLike("768B")).toBe(768);
  });

  it("should disambiguate 'B'", () => {
    // < 1024 treat as bytes
    expect(parseNumericLike("500B")).toBe(500);
    // >= 1024 or non-integer treat as billions
    expect(parseNumericLike("1024B")).toBe(1024 * 1e9);
    expect(parseNumericLike("1.5B")).toBe(1.5e9);
  });

  it("should return null for unparseable input", () => {
    expect(parseNumericLike("abc")).toBeNull();
    expect(parseNumericLike("")).toBeNull();
    expect(parseNumericLike("   ")).toBeNull();
  });
});

describe("sortData", () => {
  const data = [
    { id: 3, name: "Charlie", value: "10", date: new Date("2023-01-03"), active: true, tags: ["a", "b"] },
    { id: 1, name: "Alice", value: "2", date: new Date("2023-01-01"), active: false, tags: ["a"] },
    { id: 2, name: "Bob", value: "1", date: new Date("2023-01-02"), active: true, tags: ["a", "b", "c"] },
    { id: 4, name: "Dave", value: null, date: null, active: null, tags: null },
  ];

  it("should sort by numbers", () => {
    const sorted = sortData(data, "id", "asc");
    expect(sorted.map((d) => d.id)).toEqual([1, 2, 3, 4]);
    const desc = sortData(data, "id", "desc");
    expect(desc.map((d) => d.id)).toEqual([4, 3, 2, 1]);
  });

  it("should sort by strings", () => {
    const sorted = sortData(data, "name", "asc");
    expect(sorted.map((d) => d.name)).toEqual(["Alice", "Bob", "Charlie", "Dave"]);
  });

  it("should sort by numeric-like strings", () => {
    const sorted = sortData(data, "value", "asc");
    expect(sorted.map((d) => d.value)).toEqual(["1", "2", "10", null]);
  });

  it("should sort by dates", () => {
    const sorted = sortData(data, "date", "asc");
    expect(sorted[0].id).toBe(1);
    expect(sorted[1].id).toBe(2);
    expect(sorted[2].id).toBe(3);
    expect(sorted[3].id).toBe(4);
  });

  it("should sort by booleans", () => {
    const sorted = sortData(data, "active", "asc");
    // false < true
    expect(sorted.map((d) => d.active)).toEqual([false, true, true, null]);
  });

  it("should sort by arrays (length)", () => {
    const sorted = sortData(data, "tags", "asc");
    expect(sorted.map((d) => d.tags?.length ?? null)).toEqual([1, 2, 3, null]);
  });

  it("should handle ISO date strings", () => {
    const isoData = [
      { id: 1, d: "2023-05-01" },
      { id: 2, d: "2023-01-01" },
      { id: 3, d: "2023-03-01" },
    ];
    const sorted = sortData(isoData, "d", "asc");
    expect(sorted.map((d) => d.id)).toEqual([2, 3, 1]);
  });
});

describe("getRowIdentifier", () => {
  it("should prefer identifierKey", () => {
    const row = { id: "1", name: "Alice", custom: "CustomID" };
    expect(getRowIdentifier(row, "custom")).toBe("CustomID");
  });

  it("should fallback to name, title, then id", () => {
    expect(getRowIdentifier({ name: "Alice", id: "1" })).toBe("Alice");
    expect(getRowIdentifier({ title: "Task 1", id: "1" })).toBe("Task 1");
    expect(getRowIdentifier({ id: "1" })).toBe("1");
  });

  it("should handle arrays", () => {
    const row = { id: "1", tags: ["a", "b", null] };
    expect(getRowIdentifier(row, "tags")).toBe("a, b, null");
  });

  it("should return empty string if no identifier found", () => {
    expect(getRowIdentifier({})).toBe("");
  });
});

describe("createDataTableRowKeys", () => {
  it("should create deterministic keys based on identifier", () => {
    const rows = [
      { id: "1", name: "Alice" },
      { id: "2", name: "Bob" },
    ];
    const keys = createDataTableRowKeys(rows);
    expect(keys).toEqual(["id:Alice", "id:Bob"]);
  });

  it("should handle duplicates with disambiguation", () => {
    const rows = [
      { name: "Alice", age: 20 },
      { name: "Alice", age: 30 },
      { name: "Alice", age: 20 }, // Exact duplicate
    ];
    const keys = createDataTableRowKeys(rows);
    expect(keys[0]).toContain("id:Alice::");
    expect(keys[1]).toContain("id:Alice::");
    expect(keys[2]).toContain("id:Alice::");
    expect(keys[2]).toContain("::d2"); // Third row is duplicate of first
  });

  it("should fallback to row hash if no identifier fields exist", () => {
    const rows = [{ val: 1 }, { val: 2 }];
    const keys = createDataTableRowKeys(rows);
    expect(keys[0]).toMatch(/^row:[a-z0-9]+$/);
    expect(keys[1]).toMatch(/^row:[a-z0-9]+$/);
    expect(keys[0]).not.toBe(keys[1]);
  });
});

describe("getDataTableMobileDescriptionId", () => {
  it("should sanitize the surfaceId", () => {
    expect(getDataTableMobileDescriptionId("my-table")).toBe("my-table-mobile-table-description");
    // encodeURIComponent("%") is "%25", and the function replaces "%" with "_"
    expect(getDataTableMobileDescriptionId("table%123")).toBe("table_25123-mobile-table-description");
  });
});
