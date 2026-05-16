import { describe, expect, it } from "vitest";
import { parseSerializableDataTable, safeParseSerializableDataTable } from "./data-table-schema";

describe("parseSerializableDataTable", () => {
  it("should parse minimal valid input", () => {
    const input = {
      id: "test-table",
      columns: [{ key: "name", label: "Name" }],
      data: [{ name: "Item 1" }],
    };

    const result = parseSerializableDataTable(input);
    expect(result.id).toBe("test-table");
    expect(result.columns).toEqual([{ key: "name", label: "Name" }]);
    expect(result.data).toEqual([{ name: "Item 1" }]);
  });

  it("should parse full valid input with all optional fields", () => {
    const input = {
      id: "full-table",
      role: "information",
      receipt: {
        outcome: "success",
        summary: "Data loaded",
        at: "2023-01-01T00:00:00Z",
      },
      columns: [
        {
          key: "name",
          label: "Name",
          abbr: "N",
          sortable: true,
          align: "left",
          width: "100px",
          truncate: true,
          priority: "primary",
          hideOnMobile: false,
          format: { kind: "text" },
        },
        {
          key: "amount",
          label: "Amount",
          format: { kind: "currency", currency: "USD", decimals: 2 },
        },
      ],
      data: [
        { name: "Item 1", amount: 100 },
        { name: "Item 2", amount: 200 },
      ],
      rowIdKey: "name",
      defaultSort: { by: "name", direction: "asc" },
      sort: { by: "amount", direction: "desc" },
      emptyMessage: "No data available",
      maxHeight: "500px",
      locale: "en-US",
    };

    const result = parseSerializableDataTable(input);
    expect(result).toEqual({
      id: "full-table",
      role: "information",
      receipt: {
        outcome: "success",
        summary: "Data loaded",
        at: "2023-01-01T00:00:00Z",
      },
      columns: [
        {
          key: "name",
          label: "Name",
          abbr: "N",
          sortable: true,
          align: "left",
          width: "100px",
          truncate: true,
          priority: "primary",
          hideOnMobile: false,
          format: { kind: "text" },
        },
        {
          key: "amount",
          label: "Amount",
          format: { kind: "currency", currency: "USD", decimals: 2 },
        },
      ],
      data: [
        { name: "Item 1", amount: 100 },
        { name: "Item 2", amount: 200 },
      ],
      rowIdKey: "name",
      defaultSort: { by: "name", direction: "asc" },
      sort: { by: "amount", direction: "desc" },
      emptyMessage: "No data available",
      maxHeight: "500px",
      locale: "en-US",
    });
  });

  describe("column formats", () => {
    const baseInput = {
      id: "format-test",
      data: [],
    };

    it("should parse number format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: { kind: "number", decimals: 1, unit: "kg", compact: true, showSign: true },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "number",
        decimals: 1,
        unit: "kg",
        compact: true,
        showSign: true,
      });
    });

    it("should parse percent format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: { kind: "percent", decimals: 0, showSign: false, basis: "fraction" },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "percent",
        decimals: 0,
        showSign: false,
        basis: "fraction",
      });
    });

    it("should parse date format", () => {
      const input = {
        ...baseInput,
        columns: [{ key: "val", label: "Val", format: { kind: "date", dateFormat: "relative" } }],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({ kind: "date", dateFormat: "relative" });
    });

    it("should parse delta format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: { kind: "delta", decimals: 2, upIsPositive: true, showSign: true },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "delta",
        decimals: 2,
        upIsPositive: true,
        showSign: true,
      });
    });

    it("should parse status format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: {
              kind: "status",
              statusMap: {
                active: { tone: "success", label: "Active" },
                inactive: { tone: "neutral" },
              },
            },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "status",
        statusMap: {
          active: { tone: "success", label: "Active" },
          inactive: { tone: "neutral" },
        },
      });
    });

    it("should parse boolean format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: { kind: "boolean", labels: { true: "Yes", false: "No" } },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "boolean",
        labels: { true: "Yes", false: "No" },
      });
    });

    it("should parse link format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: { kind: "link", hrefKey: "url", external: true },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({ kind: "link", hrefKey: "url", external: true });
    });

    it("should parse badge format", () => {
      const input = {
        ...baseInput,
        columns: [
          {
            key: "val",
            label: "Val",
            format: {
              kind: "badge",
              colorMap: {
                high: "danger",
                low: "info",
              },
            },
          },
        ],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({
        kind: "badge",
        colorMap: {
          high: "danger",
          low: "info",
        },
      });
    });

    it("should parse array format", () => {
      const input = {
        ...baseInput,
        columns: [{ key: "val", label: "Val", format: { kind: "array", maxVisible: 3 } }],
      };
      const result = parseSerializableDataTable(input);
      expect(result.columns[0].format).toEqual({ kind: "array", maxVisible: 3 });
    });
  });

  describe("validation errors", () => {
    it("should throw error for missing id", () => {
      const input = {
        columns: [],
        data: [],
      };
      expect(() => parseSerializableDataTable(input)).toThrow(/id/i);
    });

    it("should throw error for missing columns", () => {
      const input = {
        id: "test",
        data: [],
      };
      expect(() => parseSerializableDataTable(input)).toThrow(/columns/i);
    });

    it("should throw error for missing data", () => {
      const input = {
        id: "test",
        columns: [],
      };
      expect(() => parseSerializableDataTable(input)).toThrow(/data/i);
    });

    it("should throw error for invalid column align", () => {
      const input = {
        id: "test",
        columns: [{ key: "name", label: "Name", align: "top" }],
        data: [],
      };
      expect(() => parseSerializableDataTable(input)).toThrow(/align/i);
    });

    it("should throw error for invalid row data (object instead of primitive)", () => {
      const input = {
        id: "test",
        columns: [{ key: "meta", label: "Meta" }],
        data: [{ meta: { some: "object" } }],
      };
      expect(() => parseSerializableDataTable(input)).toThrow(/meta/i);
    });
  });
});

describe("safeParseSerializableDataTable", () => {
  it("should return parsed data for valid input", () => {
    const input = {
      id: "test-table",
      columns: [{ key: "name", label: "Name" }],
      data: [{ name: "Item 1" }],
    };

    const result = safeParseSerializableDataTable(input);
    expect(result).not.toBeNull();
    expect(result?.id).toBe("test-table");
  });

  it("should return null for invalid input", () => {
    const input = {
      id: "", // Invalid: min(1)
      columns: [],
      data: [],
    };

    const result = safeParseSerializableDataTable(input);
    expect(result).toBeNull();
  });
});
