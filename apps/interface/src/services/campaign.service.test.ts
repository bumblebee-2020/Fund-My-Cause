import {
  filterCampaigns,
  sortCampaigns,
  searchCampaigns,
  queryCampaigns,
  getCampaignStatus,
  getCampaignProgress,
} from "./campaign.service";
import type { Campaign } from "@/types/campaign";

const mockCampaigns: Campaign[] = [
  {
    id: "1",
    contractId: "1",
    title: "Eco-Friendly Water Purification",
    description: "A water purification system",
    creator: "Alice",
    raised: 15000,
    goal: 20000,
    deadline: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString(),
    status: "Active",
    token: "XLM",
  },
  {
    id: "2",
    contractId: "2",
    title: "Solar Energy Initiative",
    description: "Community solar power",
    creator: "Bob",
    raised: 25000,
    goal: 25000,
    deadline: new Date(Date.now() + 10 * 24 * 60 * 60 * 1000).toISOString(),
    status: "Successful",
    token: "XLM",
  },
  {
    id: "3",
    contractId: "3",
    title: "Medical Research Fund",
    description: "Cancer research funding",
    creator: "Charlie",
    raised: 5000,
    goal: 50000,
    deadline: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
    status: "Refunded",
    token: "XLM",
  },
];

describe("Campaign Service", () => {
  describe("getCampaignStatus", () => {
    it("returns 'funded' when raised >= goal", () => {
      const campaign = mockCampaigns[1];
      expect(getCampaignStatus(campaign)).toBe("funded");
    });

    it("returns 'active' for ongoing campaigns", () => {
      const campaign = mockCampaigns[0];
      expect(getCampaignStatus(campaign)).toBe("active");
    });

    it("returns 'ended' when deadline has passed", () => {
      const campaign = mockCampaigns[2];
      expect(getCampaignStatus(campaign)).toBe("ended");
    });
  });

  describe("getCampaignProgress", () => {
    it("calculates progress percentage correctly", () => {
      const campaign = mockCampaigns[0];
      const progress = getCampaignProgress(campaign);
      expect(progress).toBe(75); // 15000 / 20000 * 100
    });

    it("returns 100 when raised >= goal", () => {
      const campaign = mockCampaigns[1];
      expect(getCampaignProgress(campaign)).toBe(100);
    });

    it("returns 0 when goal is 0", () => {
      const campaign = { ...mockCampaigns[0], goal: 0 };
      expect(getCampaignProgress(campaign)).toBe(0);
    });

    it("caps at 100 even if raised > goal", () => {
      const campaign = { ...mockCampaigns[0], raised: 100000, goal: 50000 };
      expect(getCampaignProgress(campaign)).toBe(100);
    });
  });

  describe("filterCampaigns", () => {
    it("returns all campaigns when filter is 'all'", () => {
      const result = filterCampaigns(mockCampaigns, "all");
      expect(result).toHaveLength(3);
    });

    it("returns only active campaigns", () => {
      const result = filterCampaigns(mockCampaigns, "active");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("1");
    });

    it("returns only funded campaigns", () => {
      const result = filterCampaigns(mockCampaigns, "funded");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("2");
    });

    it("returns only ended campaigns", () => {
      const result = filterCampaigns(mockCampaigns, "ended");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("3");
    });
  });

  describe("sortCampaigns", () => {
    it("sorts by newest (by ID descending) by default", () => {
      const result = sortCampaigns(mockCampaigns, "newest");
      expect(result[0].id).toBe("3");
      expect(result[1].id).toBe("2");
      expect(result[2].id).toBe("1");
    });

    it("sorts by most funded (progress percentage descending)", () => {
      const result = sortCampaigns(mockCampaigns, "most-funded");
      expect(result[0].id).toBe("2"); // 100%
      expect(result[1].id).toBe("1"); // 75%
      expect(result[2].id).toBe("3"); // 10%
    });

    it("sorts by ending soon (deadline ascending)", () => {
      const result = sortCampaigns(mockCampaigns, "ending-soon");
      expect(result[0].id).toBe("3"); // earliest deadline (5 days ago)
      expect(result[1].id).toBe("1"); // 5 days in future
      expect(result[2].id).toBe("2"); // 10 days in future (latest deadline)
    });
  });

  describe("searchCampaigns", () => {
    it("returns all campaigns when query is empty", () => {
      const result = searchCampaigns(mockCampaigns, "");
      expect(result).toHaveLength(3);
    });

    it("searches by title case-insensitively", () => {
      const result = searchCampaigns(mockCampaigns, "solar");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("2");
    });

    it("searches by description case-insensitively", () => {
      const result = searchCampaigns(mockCampaigns, "power");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("2");
    });

    it("searches by creator case-insensitively", () => {
      const result = searchCampaigns(mockCampaigns, "charlie");
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("3");
    });

    it("returns empty array when no matches", () => {
      const result = searchCampaigns(mockCampaigns, "nonexistent");
      expect(result).toHaveLength(0);
    });
  });

  describe("queryCampaigns", () => {
    it("applies all filters and sorts together", () => {
      const result = queryCampaigns(mockCampaigns, {
        query: "energy",
        filter: "all",
        sort: "newest",
        page: 1,
        pageSize: 10,
      });

      expect(result.results).toHaveLength(1);
      expect(result.results[0].id).toBe("2");
      expect(result.total).toBe(1);
      expect(result.totalPages).toBe(1);
    });

    it("handles pagination correctly", () => {
      const result = queryCampaigns(mockCampaigns, {
        query: "",
        filter: "all",
        sort: "newest",
        page: 1,
        pageSize: 2,
      });

      expect(result.results).toHaveLength(2);
      expect(result.total).toBe(3);
      expect(result.totalPages).toBe(2);
    });

    it("returns correct page 2", () => {
      const result = queryCampaigns(mockCampaigns, {
        query: "",
        filter: "all",
        sort: "newest",
        page: 2,
        pageSize: 2,
      });

      expect(result.results).toHaveLength(1);
      expect(result.results[0].id).toBe("1");
    });

    it("combines search, filter, and sort", () => {
      const result = queryCampaigns(mockCampaigns, {
        query: "",
        filter: "active",
        sort: "most-funded",
        page: 1,
        pageSize: 10,
      });

      expect(result.results).toHaveLength(1);
      expect(result.results[0].id).toBe("1");
    });

    it("returns at least 1 page even with no results", () => {
      const result = queryCampaigns(mockCampaigns, {
        query: "nonexistent",
        filter: "all",
        sort: "newest",
        page: 1,
        pageSize: 10,
      });

      expect(result.totalPages).toBe(1);
      expect(result.results).toHaveLength(0);
    });
  });
});
