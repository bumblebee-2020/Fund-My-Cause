"use client";

import React, { useState } from "react";
import { Loader2, RefreshCw, CheckCircle } from "lucide-react";
import { Navbar } from "@/components/layout/Navbar";
import { WalletGuard } from "@/components/WalletGuard";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { ALL_CAMPAIGNS } from "@/lib/campaigns";
import { useWallet } from "@/context/WalletContext";
import { buildRefundTx, submitSignedTx } from "@/lib/soroban";

// ── Confirmation dialog ───────────────────────────────────────────────────────

function ConfirmDialog({
  title,
  onConfirm,
  onCancel,
  pending,
}: {
  title: string;
  onConfirm: () => void;
  onCancel: () => void;
  pending: boolean;
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="refund-dialog-title"
        className="w-full max-w-sm space-y-4 rounded-2xl border border-gray-700 bg-gray-900 p-6 shadow-2xl"
      >
        <h2
          id="refund-dialog-title"
          className="text-lg font-semibold text-white"
        >
          Claim Refund
        </h2>
        <p className="text-sm text-gray-400">
          You are about to claim a refund for{" "}
          <span className="font-medium text-white">{title}</span>. This action
          cannot be undone.
        </p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            disabled={pending}
            className="px-4 py-2 text-sm text-gray-400 transition hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={pending}
            className="flex items-center gap-2 rounded-xl bg-indigo-600 px-5 py-2 text-sm font-medium transition hover:bg-indigo-500 disabled:opacity-50"
          >
            {pending && <Loader2 size={14} className="animate-spin" />}
            Confirm Claim
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Refundable campaign card ──────────────────────────────────────────────────

function RefundCard({
  campaign,
  claimed,
  onClaim,
}: {
  campaign: (typeof ALL_CAMPAIGNS)[number];
  claimed: boolean;
  onClaim: (id: string, title: string) => void;
}) {
  const progress =
    campaign.goal > 0 ? (campaign.raised / campaign.goal) * 100 : 0;

  return (
    <div className="rounded-2xl border border-gray-800 bg-gray-900 p-5 space-y-3">
      <div className="flex items-start justify-between gap-2">
        <h2 className="text-base font-semibold text-white leading-tight">
          {campaign.title}
        </h2>
        <span
          className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${
            campaign.status === "Refunded"
              ? "bg-yellow-900 text-yellow-300"
              : "bg-red-900 text-red-300"
          }`}
        >
          {campaign.status}
        </span>
      </div>

      <p className="text-sm text-gray-400 line-clamp-2">
        {campaign.description}
      </p>

      <ProgressBar progress={progress} />
      <div className="flex justify-between text-xs text-gray-500">
        <span>{campaign.raised.toLocaleString()} XLM raised</span>
        <span>Goal: {campaign.goal.toLocaleString()} XLM</span>
      </div>

      {claimed ? (
        <div className="flex items-center gap-2 rounded-xl bg-green-900/40 border border-green-700 px-4 py-2 text-sm text-green-300">
          <CheckCircle size={14} />
          Refund claimed successfully
        </div>
      ) : (
        <button
          onClick={() => onClaim(campaign.id, campaign.title)}
          className="w-full rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-500"
        >
          Claim Refund
        </button>
      )}
    </div>
  );
}

// ── Main refund page ──────────────────────────────────────────────────────────

function RefundContent() {
  const { address, signTx } = useWallet();
  const [confirmTarget, setConfirmTarget] = useState<{
    id: string;
    title: string;
  } | null>(null);
  const [pending, setPending] = useState(false);
  const [claimed, setClaimed] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  const refundableCampaigns = ALL_CAMPAIGNS.filter(
    (c) => c.status === "Refunded" || c.status === "Cancelled",
  );

  const handleConfirm = async () => {
    if (!confirmTarget || !address) return;
    setPending(true);
    setError(null);
    try {
      const xdr = await buildRefundTx(address, confirmTarget.id);
      const signed = await signTx(xdr);
      await submitSignedTx(signed);
      setClaimed((prev) => new Set(prev).add(confirmTarget.id));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Transaction failed.");
    } finally {
      setPending(false);
      setConfirmTarget(null);
    }
  };

  return (
    <div className="mx-auto max-w-4xl px-6 py-12">
      <h1 className="text-3xl font-bold mb-2">Claim Refunds</h1>
      <p className="text-sm text-gray-400 mb-8">
        Campaigns below were cancelled or refunded. If you contributed, you may
        claim your funds back.
      </p>

      {error && (
        <div className="mb-6 rounded-xl border border-red-700 bg-red-900/40 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      {refundableCampaigns.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-24 text-center">
          <RefreshCw size={40} className="text-gray-600 mb-4" />
          <p className="text-lg font-medium text-gray-400">
            No refundable campaigns
          </p>
          <p className="text-sm text-gray-600 mt-1">
            Campaigns you contributed to that were cancelled or refunded will
            appear here.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
          {refundableCampaigns.map((campaign) => (
            <RefundCard
              key={campaign.id}
              campaign={campaign}
              claimed={claimed.has(campaign.id)}
              onClaim={(id, title) => setConfirmTarget({ id, title })}
            />
          ))}
        </div>
      )}

      {confirmTarget && (
        <ConfirmDialog
          title={confirmTarget.title}
          pending={pending}
          onConfirm={handleConfirm}
          onCancel={() => setConfirmTarget(null)}
        />
      )}
    </div>
  );
}

export default function RefundPage() {
  return (
    <main className="min-h-screen bg-gray-950 text-white">
      <Navbar />
      <WalletGuard message="Connect your wallet to claim refunds.">
        <RefundContent />
      </WalletGuard>
    </main>
  );
}
