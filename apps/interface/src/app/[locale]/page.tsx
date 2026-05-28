import React from "react";
import Link from "next/link";
import { Navbar } from "@/components/layout/Navbar";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { RecommendedSection } from "@/components/ui/RecommendedSection";
import { Rocket, Users, Coins, ArrowRight, PlusCircle } from "lucide-react";
import { formatTimeLeft } from "@/lib/format";
import {
  DEFAULT_CAMPAIGN_IMAGE,
  DEFAULT_CAMPAIGN_IMAGE_ALT_1,
  DEFAULT_CAMPAIGN_IMAGE_ALT_2,
} from "@/lib/constants";
import { useTranslations } from "next-intl";

// ── Data ──────────────────────────────────────────────────────────────────────

const FEATURED = [
  {
    id: "1",
    title: "Eco-Friendly Water Purification",
    description:
      "A compact, solar-powered water purification system for off-grid communities.",
    raised: 15400,
    goal: 20000,
    deadline: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString(),
    image: DEFAULT_CAMPAIGN_IMAGE,
    featured: true,
  },
  {
    id: "2",
    title: "Open Source AI Education Platform",
    description:
      "Democratizing AI education with free, high-quality interactive courses for everyone.",
    raised: 8200,
    goal: 50000,
    deadline: new Date(Date.now() + 12 * 24 * 60 * 60 * 1000).toISOString(),
    image: DEFAULT_CAMPAIGN_IMAGE_ALT_1,
    featured: true,
  },
  {
    id: "3",
    title: "Community Solar Microgrid",
    description:
      "Empowering neighborhoods to generate and share sustainable solar energy.",
    raised: 45000,
    goal: 45000,
    deadline: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString(),
    image: DEFAULT_CAMPAIGN_IMAGE_ALT_2,
    featured: true,
  },
];

// ── Page ──────────────────────────────────────────────────────────────────────

export default function Home() {
  const t = useTranslations("home");

  const STATS = [
    { label: t("stats.campaignsLaunched"), value: "128", icon: Rocket },
    { label: t("stats.totalRaised"), value: "2.4M XLM", icon: Coins },
    { label: t("stats.contributors"), value: "9,300+", icon: Users },
  ];

  const HOW_IT_WORKS = [
    {
      step: 1,
      title: t("steps.createTitle"),
      description: t("steps.createDesc"),
    },
    { step: 2, title: t("steps.fundTitle"), description: t("steps.fundDesc") },
    {
      step: 3,
      title: t("steps.withdrawTitle"),
      description: t("steps.withdrawDesc"),
    },
  ];

  return (
    <main
      id="main-content"
      className="min-h-screen bg-gray-950 text-white flex flex-col"
    >
      <Navbar />

      {/* ── Hero ── */}
      <section className="max-w-4xl mx-auto px-6 py-24 text-center">
        <div className="inline-flex items-center gap-2 bg-indigo-950 border border-indigo-800 text-indigo-300 text-xs font-medium px-3 py-1 rounded-full mb-6">
          <Rocket size={12} /> {t("badge")}
        </div>
        <h1 className="text-5xl sm:text-6xl font-bold leading-tight mb-5 whitespace-pre-line">
          {t("heroTitle")}
        </h1>
        <p className="text-gray-400 text-lg max-w-xl mx-auto mb-8">
          {t("heroSubtitle")}
        </p>
        <div className="flex flex-wrap justify-center gap-3">
          <Link
            href="/campaigns"
            className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 px-6 py-3 rounded-xl font-medium transition"
          >
            {t("exploreCampaigns")} <ArrowRight size={16} />
          </Link>
          <Link
            href="/create"
            className="flex items-center gap-2 bg-gray-800 hover:bg-gray-700 px-6 py-3 rounded-xl font-medium transition"
          >
            <PlusCircle size={16} /> {t("startCampaign")}
          </Link>
        </div>
      </section>

      {/* ── Stats bar ── */}
      <section className="border-y border-gray-800 bg-gray-900/50">
        <div className="max-w-4xl mx-auto px-6 py-8 grid grid-cols-1 sm:grid-cols-3 gap-6 text-center">
          {STATS.map(({ label, value, icon: Icon }) => (
            <div key={label} className="flex flex-col items-center gap-2">
              <Icon size={20} className="text-indigo-400" />
              <span className="text-2xl font-bold">{value}</span>
              <span className="text-sm text-gray-400">{label}</span>
            </div>
          ))}
        </div>
      </section>

      {/* ── Featured campaigns ── */}
      <section className="max-w-6xl mx-auto px-6 py-16">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold">{t("featuredCampaigns")}</h2>
          <Link
            href="/campaigns"
            className="text-sm text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition"
          >
            {t("viewAll")} <ArrowRight size={14} />
          </Link>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {FEATURED.map((c) => {
            const progress = (c.raised / c.goal) * 100;
            const isFunded = progress >= 100;
            const deadlineSecs = Math.floor(
              new Date(c.deadline).getTime() / 1000,
            );
            return (
              <div
                key={c.id}
                className="relative bg-gray-900 rounded-2xl overflow-hidden border border-gray-800"
              >
                {c.featured && (
                  <span className="absolute top-3 left-3 z-10 flex items-center gap-1 bg-yellow-500/90 text-yellow-950 text-xs font-semibold px-2 py-0.5 rounded-full">
                    <span aria-hidden="true">⭐</span> {t("featured")}
                  </span>
                )}
                <img
                  src={c.image}
                  alt={c.title}
                  className="w-full h-48 object-cover"
                />
                <div className="p-5 space-y-3">
                  <h3 className="text-base font-semibold">{c.title}</h3>
                  <p className="text-gray-400 text-sm">{c.description}</p>
                  <ProgressBar progress={progress} />
                  <div className="flex justify-between text-sm text-gray-400">
                    <span>
                      {t("raised", { amount: c.raised.toLocaleString() })}
                    </span>
                    <span>
                      {t("goal", { amount: c.goal.toLocaleString() })}
                    </span>
                  </div>
                  <p className="text-xs text-gray-500">
                    {t("timeLeft", { time: formatTimeLeft(deadlineSecs) })}
                  </p>
                  <Link
                    href="/campaigns"
                    aria-label={`View and pledge to ${c.title}`}
                    className="block w-full py-2 rounded-xl font-medium text-center bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 transition"
                  >
                    {isFunded ? t("successfullyFunded") : t("pledgeNow")}
                  </Link>
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* ── How It Works ── */}
      <RecommendedSection />

      <section className="bg-gray-900/50 border-t border-gray-800">
        <div className="max-w-4xl mx-auto px-6 py-16 text-center">
          <h2 className="text-2xl font-bold mb-10">{t("howItWorks")}</h2>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-8">
            {HOW_IT_WORKS.map(({ step, title, description }) => (
              <div key={step} className="flex flex-col items-center gap-3">
                <div className="w-10 h-10 rounded-full bg-indigo-600 flex items-center justify-center font-bold text-sm">
                  {step}
                </div>
                <h3 className="font-semibold text-lg">{title}</h3>
                <p className="text-gray-400 text-sm">{description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Footer ── */}
      <footer className="border-t border-gray-800 mt-auto">
        <div className="max-w-6xl mx-auto px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-gray-500">
          <div className="flex items-center gap-2 font-semibold text-white">
            <Rocket size={16} className="text-indigo-400" /> Fund-My-Cause
          </div>
          <div className="flex gap-5">
            <Link href="/campaigns" className="hover:text-white transition">
              {t("footer.campaigns")}
            </Link>
            <Link href="/create" className="hover:text-white transition">
              {t("footer.create")}
            </Link>
            <Link href="/dashboard" className="hover:text-white transition">
              {t("footer.dashboard")}
            </Link>
            <Link href="/bookmarks" className="hover:text-white transition">
              {t("footer.bookmarks")}
            </Link>
            <a
              href="https://developers.stellar.org"
              target="_blank"
              rel="noreferrer"
              className="hover:text-white transition"
            >
              {t("footer.stellarDocs")}
            </a>
          </div>
          <span>{t("footer.license")}</span>
        </div>
      </footer>
    </main>
  );
}
