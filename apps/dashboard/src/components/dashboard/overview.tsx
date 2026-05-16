"use client";

import * as React from "react";
import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useTransactionSummary } from "@/hooks/use-transactions";

export function Overview() {
  const { summary, isLoading } = useTransactionSummary();

  const chartData = React.useMemo(() => {
    if (!summary) return [];
    return summary.monthly_trends.map((t) => ({
      name: t.month,
      total: parseFloat(t.expense),
    }));
  }, [summary]);

  if (isLoading) {
    return <div className="h-[350px] w-full animate-pulse bg-muted rounded-xl" />;
  }

  return (
    <ResponsiveContainer width="100%" height={350}>
      <BarChart data={chartData}>
        <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
        <YAxis
          direction="ltr"
          stroke="#888888"
          fontSize={12}
          tickLine={false}
          axisLine={false}
          tickFormatter={(value) => `₹${value}`}
        />
        <Tooltip cursor={{ fill: "var(--muted)" }} contentStyle={{ borderRadius: "8px" }} />
        <Bar dataKey="total" fill="currentColor" radius={[4, 4, 0, 0]} className="fill-primary" />
      </BarChart>
    </ResponsiveContainer>
  );
}
