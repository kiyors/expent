"use client";

import { BellIcon, ChevronRightIcon, MonitorIcon, PaletteIcon, TagIcon, UserCogIcon, WrenchIcon } from "lucide-react";
import { m, type Variants } from "motion/react";
import Link from "next/link";

const sidebarNavItems = [
  {
    title: "Profile",
    description: "Manage your public profile, username, and avatar.",
    href: "/settings/profile",
    icon: <UserCogIcon className="size-5" />,
    color: "text-blue-500",
    bg: "bg-blue-500/10",
  },
  {
    title: "Account",
    description: "Update your email, password, and core security settings.",
    href: "/settings/account",
    icon: <WrenchIcon className="size-5" />,
    color: "text-indigo-500",
    bg: "bg-indigo-500/10",
  },
  {
    title: "Categories",
    description: "Manage custom categories for tagging your transactions.",
    href: "/settings/categories",
    icon: <TagIcon className="size-5" />,
    color: "text-emerald-500",
    bg: "bg-emerald-500/10",
  },
  {
    title: "Appearance",
    description: "Customize the theme, layout, and colors of the dashboard.",
    href: "/settings/appearance",
    icon: <PaletteIcon className="size-5" />,
    color: "text-pink-500",
    bg: "bg-pink-500/10",
  },
  {
    title: "Notifications",
    description: "Configure how and when you receive alerts and updates.",
    href: "/settings/notifications",
    icon: <BellIcon className="size-5" />,
    color: "text-amber-500",
    bg: "bg-amber-500/10",
  },
  {
    title: "Display",
    description: "Change data formatting, currency formatting, and date styles.",
    href: "/settings/display",
    icon: <MonitorIcon className="size-5" />,
    color: "text-cyan-500",
    bg: "bg-cyan-500/10",
  },
];

const containerVariants: Variants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants: Variants = {
  hidden: { opacity: 0, y: 15, scale: 0.95 },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: {
      type: "spring",
      stiffness: 300,
      damping: 24,
    },
  },
};

export default function SettingsIndexPage() {
  return (
    <div className="w-full max-w-4xl pb-10 mx-auto">
      <div className="mb-8 gap-y-2 text-center">
        <h2 className="text-3xl font-semibold tracking-tight">Overview</h2>
        <p className="text-muted-foreground text-base">Select a category to view and manage your workspace settings.</p>
      </div>

      <m.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="grid gap-4 sm:grid-cols-2 md:gap-6"
      >
        {sidebarNavItems.map((item) => (
          <m.div
            key={item.href}
            variants={itemVariants}
            whileHover={{ y: -4 }}
            whileTap={{ scale: 0.97 }}
            className="h-full"
          >
            <Link
              href={item.href}
              transitionTypes={["nav-forward"]}
              className="group flex flex-col justify-between h-full p-6 rounded-2xl border bg-card/60 backdrop-blur-md text-card-foreground shadow-sm hover:shadow-md transition-all duration-300 relative overflow-hidden ring-1 ring-inset ring-foreground/5 hover:ring-primary/20"
            >
              {/* Subtle background glow effect */}
              <div className="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 rounded-2xl" />

              <div className="relative z-10">
                <div className="flex items-start justify-between mb-5">
                  <div
                    className={`p-3 rounded-xl ${item.bg} ${item.color} group-hover:scale-110 group-hover:shadow-sm transition-all duration-300`}
                  >
                    {item.icon}
                  </div>
                  <div className="size-8 rounded-full bg-muted/40 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-foreground text-muted-foreground transition-colors duration-300">
                    <ChevronRightIcon className="size-4" />
                  </div>
                </div>

                <h3 className="text-lg font-semibold mb-2 tracking-tight group-hover:text-primary transition-colors">
                  {item.title}
                </h3>
                <p className="text-sm text-muted-foreground leading-relaxed">{item.description}</p>
              </div>
            </Link>
          </m.div>
        ))}
      </m.div>
    </div>
  );
}
