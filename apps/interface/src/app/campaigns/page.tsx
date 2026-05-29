"use client";

import React, { Suspense, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Link from "next/link";
import { Navbar } from "@/components/layout/Navbar";
import { CampaignCard } from "@/components/ui/CampaignCard";
import { PledgeModal } from "@/components/ui/PledgeModal";
import {
  EmptyState,
  NoCampaignsIllustration,
} from "@/components/ui/EmptyState";
import { LoadingSkeletonGrid } from "@/components/ui/LoadingSkeleton";
import { Campaign } from "@/types/campaign";
import { ALL_CAMPAIGNS } from "@/lib/campaigns";
import { Search, GitCompare, SlidersHorizontal, X } from "lucide-react";
import { useComparison } from "@/context/ComparisonContext";
import { Pagination } from "@/components/ui/Pagination";
import { ShareModal } from "@/components/ui/ShareModal";

// ── Types ─────────────────────────────────────────────────────────────────────

type FilterTab = "all" | "active" | "funded" | "ended";
type SortOption = "recent" | "most-funded" | "ending-soon" | "progress";

// ── Constants ─────────────────────────────────────────────────────────────────

const CATEGORIES = ["Technology", "Education", "Health", "Arts", "Environment", "Community"];
const SORT_OPTIONS = [
  { label: "Newest", value: "recent" },
  { label: "Most Funded", value: "most-funded" },
  { label: "Ending Soon", value: "ending-soon" },
];

const selectCls = "bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-xl px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:border-indigo-500";
const inputCls = "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500";

// ── Helpers ───────────────────────────────────────────────────────────────────

function getStatus(c: Campaign): FilterTab {
  const ended = new Date(c.deadline) < new Date();
  if (c.raised >= c.goal) return "funded";
  if (ended) return "ended";
  return "active";
}

function applyFilter(campaigns: Campaign[], filter: FilterTab): Campaign[] {
  if (filter === "all") return campaigns;
  return campaigns.filter((c) => getStatus(c) === filter);
}

function applySort(campaigns: Campaign[], sort: SortOption): Campaign[] {
  return [...campaigns].sort((a, b) => {
    if (sort === "most-funded") return b.raised / b.goal - a.raised / a.goal;
    if (sort === "ending-soon")
      return new Date(a.deadline).getTime() - new Date(b.deadline).getTime();
    return Number(b.id) - Number(a.id); // "recent" - newest first
  });
}

const FILTER_TABS: { label: string; value: FilterTab }[] = [
  { label: "All", value: "all" },
  { label: "Active", value: "active" },
  { label: "Funded", value: "funded" },
  { label: "Ended", value: "ended" },
];

const PAGE_SIZE_OPTIONS = [9, 18, 36];

// ── Inner component (uses useSearchParams) ────────────────────────────────────

export function CampaignsInner() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { selected, clear } = useComparison();

  const filter = (searchParams.get("filter") as FilterTab) ?? "all";
  const sort = (searchParams.get("sort") as SortOption) ?? "recent";
  const query = searchParams.get("q") ?? "";
  const category = searchParams.get("category") ?? "";
  const page = Math.max(1, Number(searchParams.get("page") ?? "1"));

  const pageSize = Math.max(
    1,
    Number(searchParams.get("pageSize") ?? String(PAGE_SIZE_OPTIONS[0])),
  );

  const [pledge, setPledge] = useState<string | null>(null);
  const [shareTarget, setShareTarget] = useState<{
    id: string;
    title: string;
  } | null>(null);
  const [inputValue, setInputValue] = useState(query);
  const searchQuery = inputValue.trim();
  const hasSearch = searchQuery.length > 0;
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [goalMin, setGoalMin] = useState(searchParams.get("goalMin") ?? "");
  const [goalMax, setGoalMax] = useState(searchParams.get("goalMax") ?? "");
  const [dateFrom, setDateFrom] = useState(searchParams.get("dateFrom") ?? "");
  const [dateTo, setDateTo] = useState(searchParams.get("dateTo") ?? "");

  const setParam = (key: string, value: string) => {
    const params = new URLSearchParams(searchParams.toString());
    if (
      value === "" ||
      (key === "filter" && value === "all") ||
      (key === "sort" && value === "recent")
    ) {
      params.delete(key);
    } else {
      params.set(key, value);
    }
    if (key !== "page") params.delete("page");
    router.replace(`/campaigns?${params.toString()}`, { scroll: false });
  };

  React.useEffect(() => {
    setInputValue(query);
  }, [query]);

  React.useEffect(() => {
    const timer = setTimeout(() => setParam("q", inputValue), 300);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inputValue]);

  const setPage = (p: number) => {
    const params = new URLSearchParams(searchParams.toString());
    if (p === 1) params.delete("page");
    else params.set("page", String(p));
    router.replace(`/campaigns?${params.toString()}`, { scroll: false });
  };

  const setPageSize = (size: number) => {
    const params = new URLSearchParams(searchParams.toString());
    params.set("pageSize", String(size));
    params.delete("page");
    router.replace(`/campaigns?${params.toString()}`, { scroll: false });
  };

  const clearAdvanced = () => {
    setGoalMin("");
    setGoalMax("");
    setDateFrom("");
    setDateTo("");
    const params = new URLSearchParams(searchParams.toString());
    params.delete("goalMin");
    params.delete("goalMax");
    params.delete("dateFrom");
    params.delete("dateTo");
    params.delete("page");
    router.replace(`/campaigns?${params.toString()}`, { scroll: false });
  };

  const applyAdvanced = () => {
    const params = new URLSearchParams(searchParams.toString());
    if (goalMin) params.set("goalMin", goalMin);
    else params.delete("goalMin");
    if (goalMax) params.set("goalMax", goalMax);
    else params.delete("goalMax");
    if (dateFrom) params.set("dateFrom", dateFrom);
    else params.delete("dateFrom");
    if (dateTo) params.set("dateTo", dateTo);
    else params.delete("dateTo");
    params.delete("page");
    router.replace(`/campaigns?${params.toString()}`, { scroll: false });
    setShowAdvanced(false);
  };

  const activeGoalMin = searchParams.get("goalMin")
    ? Number(searchParams.get("goalMin"))
    : null;
  const activeGoalMax = searchParams.get("goalMax")
    ? Number(searchParams.get("goalMax"))
    : null;
  const activeDateFrom = searchParams.get("dateFrom") ?? null;
  const activeDateTo = searchParams.get("dateTo") ?? null;
  const hasAdvanced = !!(
    activeGoalMin ||
    activeGoalMax ||
    activeDateFrom ||
    activeDateTo
  );

  const filtered = applySort(
    applyFilter(
      ALL_CAMPAIGNS.filter((c) => {
        if (hasSearch) {
          const q = searchQuery.toLowerCase();
          if (
            !c.title.toLowerCase().includes(q) &&
            !c.description.toLowerCase().includes(q) &&
            !c.creator.toLowerCase().includes(q)
          )
            return false;
        }
        if (category && c.category !== category) return false;
        if (activeGoalMin !== null && c.goal < activeGoalMin) return false;
        if (activeGoalMax !== null && c.goal > activeGoalMax) return false;
        if (activeDateFrom && new Date(c.deadline) < new Date(activeDateFrom))
          return false;
        if (activeDateTo && new Date(c.deadline) > new Date(activeDateTo))
          return false;
        return true;
      }),
      filter,
    ),
    sort,
  );

  const totalPages = Math.max(1, Math.ceil(filtered.length / pageSize));
  const currentPage = Math.min(page, totalPages);
  const paginated = filtered.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize,
  );
  const resultStart = filtered.length === 0 ? 0 : (currentPage - 1) * pageSize + 1;
  const resultEnd = Math.min(filtered.length, currentPage * pageSize);

  return (
    <>
      {/* Controls row */}
      <div className="flex flex-col sm:flex-row gap-3 mb-4">
        <div className="relative flex-1">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
          />
          <input
            type="text"
            placeholder="Search campaigns..."
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            aria-label="Search campaigns"
            className="w-full bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-xl pl-9 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-indigo-500"
          />
        </div>

        {/* Category filter */}
        <select
          value={category}
          onChange={(e) => setParam("category", e.target.value)}
          aria-label="Filter by category"
          className={selectCls}
        >
          <option value="">All Categories</option>
          {CATEGORIES.map((cat) => (
            <option key={cat} value={cat}>
              {cat.charAt(0).toUpperCase() + cat.slice(1)}
            </option>
          ))}
        </select>

        {/* Sort */}
        <select
          value={sort}
          onChange={(e) => setParam("sort", e.target.value)}
          aria-label="Sort campaigns"
          className={selectCls}
        >
          {SORT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>

        <button
          onClick={() => setShowMobileFilters((value) => !value)}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium transition bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-indigo-500"
        >
          <SlidersHorizontal size={14} />
          Filters & Sort
        </button>

        {/* Advanced filters toggle */}
        <button
          onClick={() => setShowAdvanced((v) => !v)}
          aria-expanded={showAdvanced}
          className={`flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium transition border ${
            hasAdvanced
              ? "bg-indigo-600 border-indigo-500 text-white"
              : "bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-indigo-500"
          }`}
        >
          <SlidersHorizontal size={14} />
          Filters{hasAdvanced ? " ●" : ""}
        </button>

        {hasSearch && (
          <button
            onClick={() => {
              setInputValue("");
              setParam("q", "");
            }}
            className="text-sm text-indigo-600 hover:text-indigo-500 transition"
          >
            Clear search
          </button>
        )}
      </div>

      {/* Advanced filter panel */}
      {showAdvanced && (
        <div className="mb-4 rounded-2xl border border-gray-700 bg-gray-900 p-5 space-y-4">
          <h3 className="text-sm font-semibold text-gray-300">
            Advanced Filters
          </h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="mb-1 block text-xs text-gray-400">
                Min Goal (XLM)
              </label>
              <input
                type="number"
                min={0}
                value={goalMin}
                onChange={(e) => setGoalMin(e.target.value)}
                placeholder="e.g. 1000"
                className={inputCls}
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-gray-400">
                Max Goal (XLM)
              </label>
              <input
                type="number"
                min={0}
                value={goalMax}
                onChange={(e) => setGoalMax(e.target.value)}
                placeholder="e.g. 50000"
                className={inputCls}
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-gray-400">
                Deadline From
              </label>
              <input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
                className={inputCls}
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-gray-400">
                Deadline To
              </label>
              <input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
                className={inputCls}
              />
            </div>
          </div>
          <div className="flex gap-2 justify-end">
            <button
              onClick={clearAdvanced}
              className="flex items-center gap-1 px-3 py-1.5 text-xs text-gray-400 hover:text-white transition"
            >
              <X size={12} /> Clear
            </button>
            <button
              onClick={applyAdvanced}
              className="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-sm font-medium text-white transition"
            >
              Apply
            </button>
          </div>
        </div>
      )}
      {showMobileFilters && (
        <div className="mb-4 rounded-2xl border border-gray-700 bg-gray-900 p-5 space-y-4 sm:hidden">
          <h3 className="text-sm font-semibold text-gray-300">Filter by Status</h3>
          <div className="flex flex-col gap-2">
            {FILTER_TABS.map((tab, idx) => (
              <button
                key={tab.value}
                onClick={() => setParam("filter", tab.value)}
                className={`w-full text-left rounded-xl px-4 py-3 text-sm font-medium transition ${
                  filter === tab.value
                    ? "bg-indigo-600 text-white"
                    : "bg-gray-800 text-gray-200 hover:bg-gray-700"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Filter tabs */}
      <div
        className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between mb-6"
      >
        <div className="flex flex-wrap gap-2" role="tablist" aria-label="Filter campaigns">
        {FILTER_TABS.map((tab, idx) => (
          <button
            key={tab.value}
            role="tab"
            aria-selected={filter === tab.value}
            tabIndex={filter === tab.value ? 0 : -1}
            onClick={() => setParam("filter", tab.value)}
            onKeyDown={(e) => {
              if (e.key === "ArrowRight") {
                const next = FILTER_TABS[(idx + 1) % FILTER_TABS.length];
                setParam("filter", next.value);
                (
                  e.currentTarget.parentElement?.children[
                    (idx + 1) % FILTER_TABS.length
                  ] as HTMLElement
                )?.focus();
              } else if (e.key === "ArrowLeft") {
                const prev =
                  FILTER_TABS[
                    (idx - 1 + FILTER_TABS.length) % FILTER_TABS.length
                  ];
                setParam("filter", prev.value);
                (
                  e.currentTarget.parentElement?.children[
                    (idx - 1 + FILTER_TABS.length) % FILTER_TABS.length
                  ] as HTMLElement
                )?.focus();
              }
            }}
            className={`px-4 py-1.5 rounded-full text-sm font-medium transition ${
              filter === tab.value
                ? "bg-indigo-600 text-white"
                : "bg-gray-200 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
      </div>

      {filtered.length > 0 && (
        <p className="text-sm text-gray-500 mb-4">
          Showing {resultStart}–{resultEnd} of {filtered.length} campaign
          {filtered.length === 1 ? "" : "s"}
        </p>
      )}

      {/* Grid */}
      {filtered.length === 0 ? (
        <EmptyState
          illustration={<NoCampaignsIllustration />}
          title="No campaigns found"
          description="Try adjusting your search or filters to find what you're looking for."
          action={{
            label: "Browse All Campaigns",
            onClick: () => router.replace("/campaigns"),
          }}
        />
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {paginated.map((campaign, i) => (
              <CampaignCard
                key={campaign.id}
                campaign={campaign}
                onPledge={(id) => setPledge(id)}
                onShare={(id, title) => setShareTarget({ id, title })}
                index={i}
                query={query}
              />
            ))}
          </div>

          <Pagination
            currentPage={currentPage}
            totalPages={totalPages}
            totalItems={filtered.length}
            pageSize={pageSize}
            onPageChange={setPage}
            onPageSizeChange={setPageSize}
            pageSizeOptions={PAGE_SIZE_OPTIONS}
          />
        </>
      )}

      {pledge && (
        <PledgeModal
          campaignTitle={
            ALL_CAMPAIGNS.find((c) => c.id === pledge)?.title ?? pledge
          }
          onClose={() => setPledge(null)}
        />
      )}

      {shareTarget && (
        <ShareModal
          campaignId={shareTarget.id}
          campaignTitle={shareTarget.title}
          onClose={() => setShareTarget(null)}
        />
      )}

      {/* Comparison floating bar */}
      {selected.length > 0 && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 bg-gray-900 border border-gray-700 rounded-2xl px-5 py-3 shadow-2xl">
          <GitCompare size={16} className="text-indigo-400" />
          <span className="text-sm text-gray-300">
            {selected.length} selected
          </span>
          <Link
            href="/compare"
            className="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-sm font-medium transition"
          >
            Compare
          </Link>
          <button
            onClick={clear}
            className="text-gray-500 hover:text-gray-300 text-xs transition"
          >
            Clear
          </button>
        </div>
      )}
    </>
  );
}

// ── Page ──────────────────────────────────────────────────────────────────────

export default function DiscoveryPage() {
  return (
    <main className="min-h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-white">
      <Navbar />
      <div className="max-w-6xl mx-auto px-6 py-12">
        <h1 className="text-3xl font-bold mb-2">Discover Campaigns</h1>
        <p className="text-gray-500 mb-8">
          Find and support causes that matter to you
        </p>
        <Suspense fallback={<LoadingSkeletonGrid count={6} />}>
          <CampaignsInner />
        </Suspense>
      </div>
    </main>
  );
}
