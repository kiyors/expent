"use client";

import * as React from "react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { useTransactionSummary } from "@/hooks/use-transactions";

const COLORS = [
  "#3b82f6",
  "#ef4444",
  "#10b981",
  "#f97316",
  "#8b5cf6",
  "#ec4899",
  "#06b6d4",
  "#eab308",
  "#64748b",
  "#14b8a6",
];

export function CategoryChart() {
  const { summary, isLoading } = useTransactionSummary();

  const data = React.useMemo(() => {
    if (!summary) return [];
    return summary.category_distribution.map((c) => ({
      name: c.name,
      value: Math.round(parseFloat(c.amount)),
    }));
  }, [summary]);

  if (isLoading) {
    return <div className="h-[300px] w-full animate-pulse bg-muted rounded-xl" />;
  }

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[300px] text-muted-foreground text-sm">
        No expense data to display.
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={300}>
      <PieChart>
        <Pie
          data={data}
          cx="50%"
          cy="50%"
          innerRadius={60}
          outerRadius={100}
          paddingAngle={4}
          dataKey="value"
          label={({ name, percent }) => `${name} ${((percent ?? 0) * 100).toFixed(0)}%`}
          labelLine={false}
        >
          {data.map((entry, index) => (
            <Cell key={`cell-${index}-${entry.name}`} fill={COLORS[index % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip contentStyle={{ borderRadius: "8px" }} />
      </PieChart>
    </ResponsiveContainer>
  );
}
