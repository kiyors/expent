"use client";

import * as React from "react";
import { Bar, BarChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useTransactionSummary } from "@/hooks/use-transactions";

export function IncomeExpenseChart() {
  const { summary, isLoading } = useTransactionSummary();

  const chartData = React.useMemo(() => {
    if (!summary) return [];
    return summary.monthly_trends.map((t) => ({
      name: t.month,
      income: parseFloat(t.income),
      expense: parseFloat(t.expense),
    }));
  }, [summary]);

  if (isLoading) {
    return <div className="h-[300px] w-full animate-pulse bg-muted rounded-xl" />;
  }

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
        <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
        <YAxis stroke="#888888" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(v) => `₹${v}`} />
        <Tooltip contentStyle={{ borderRadius: "8px" }} />
        <Legend />
        <Bar dataKey="income" fill="#10b981" radius={[4, 4, 0, 0]} name="Income" />
        <Bar dataKey="expense" fill="#ef4444" radius={[4, 4, 0, 0]} name="Expense" />
      </BarChart>
    </ResponsiveContainer>
  );
}
