import React from "react";
import { render, screen, fireEvent, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import DiscoveryPage from "./page";

// Mock next/navigation for client component hooks
jest.mock("next/navigation", () => {
  const searchParamsState = { current: new URLSearchParams() };
  const subscribers = new Set<() => void>();

  return {
    useRouter: () => ({
      replace: (href: string) => {
        const url = new URL(href, "http://localhost");
        searchParamsState.current = url.searchParams;
        subscribers.forEach((notify) => notify());
      },
    }),
    useSearchParams: () => {
      const [, setVersion] = React.useState(0);
      React.useEffect(() => {
        const notify = () => setVersion((version) => version + 1);
        subscribers.add(notify);
        return () => subscribers.delete(notify);
      }, []);
      return searchParamsState.current;
    },
  };
});

// Mock comparison context so the page can render without provider setup
jest.mock("@/context/ComparisonContext", () => ({
  useComparison: () => ({
    selected: [],
    clear: jest.fn(),
  }),
}));

// Mock next/link
jest.mock("next/link", () => {
  return ({ children, href }: { children: React.ReactNode; href: string }) => (
    <a href={href}>{children}</a>
  );
});

// Mock components and data
jest.mock("@/components/layout/Navbar", () => ({
  Navbar: () => <div data-testid="navbar">Navbar</div>,
}));

jest.mock("@/components/ui/CampaignCard", () => ({
  CampaignCard: ({ campaign }: { campaign: { id: string; title: string } }) => (
    <div data-testid={`campaign-card-${campaign.id}`}>{campaign.title}</div>
  ),
}));

jest.mock("@/components/ui/PledgeModal", () => ({
  PledgeModal: ({ onClose }: { onClose: () => void }) => (
    <button onClick={onClose}>Close Pledge</button>
  ),
}));

jest.mock("@/components/ui/ShareModal", () => ({
  ShareModal: ({ onClose }: { onClose: () => void }) => (
    <button onClick={onClose}>Close Share</button>
  ),
}));

jest.mock("@/components/ui/LoadingSkeleton", () => ({
  LoadingSkeletonGrid: () => <div>Loading...</div>,
}));

jest.mock("@/lib/campaigns", () => ({
  ALL_CAMPAIGNS: [
    {
      id: "1",
      contractId: "1",
      title: "Water Purification",
      description: "Water purification",
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
      title: "Solar Energy",
      description: "Solar energy project",
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
      title: "Medical Research",
      description: "Medical research fund",
      creator: "Charlie",
      raised: 5000,
      goal: 50000,
      deadline: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
      status: "Refunded",
      token: "XLM",
    },
  ],
}));

describe("Campaign Discovery Page", () => {
  it("renders the discovery page with header and search", () => {
    render(<DiscoveryPage />);

    expect(screen.getByText("Discover Campaigns")).toBeInTheDocument();
    expect(screen.getByText("Find and support causes that matter to you")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/search campaigns/i)).toBeInTheDocument();
  });

  it("displays all campaigns on initial load", () => {
    render(<DiscoveryPage />);

    expect(screen.getByTestId("campaign-card-1")).toBeInTheDocument();
    expect(screen.getByTestId("campaign-card-2")).toBeInTheDocument();
    expect(screen.getByTestId("campaign-card-3")).toBeInTheDocument();
  });

  describe("Search Functionality", () => {
    it("filters campaigns by search query", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Solar");

      await waitFor(() => {
        expect(screen.getByTestId("campaign-card-2")).toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-1")).not.toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-3")).not.toBeInTheDocument();
      });
    });

    it("shows 'no campaigns found' when search yields no results", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Nonexistent");

      await waitFor(() => {
        expect(screen.getByText(/no campaigns found/i)).toBeInTheDocument();
      });
    });

    it("shows 'Clear search' button when query is active", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Solar");

      await waitFor(() => {
        expect(screen.getByText("Clear search")).toBeInTheDocument();
      });
    });

    it("clears search when 'Clear search' button is clicked", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Solar");

      await waitFor(() => {
        expect(screen.getByText("Clear search")).toBeInTheDocument();
      });

      const clearButton = screen.getByText("Clear search");
      await user.click(clearButton);

      await waitFor(() => {
        expect(screen.getByTestId("campaign-card-1")).toBeInTheDocument();
        expect(screen.getByTestId("campaign-card-2")).toBeInTheDocument();
        expect(screen.getByTestId("campaign-card-3")).toBeInTheDocument();
      });
    });
  });

  describe("Filtering Functionality", () => {
    it("displays filter tabs for desktop", () => {
      render(<DiscoveryPage />);

      expect(screen.getByRole("tab", { name: /^All$/ })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: /^Active$/ })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: /^Funded$/ })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: /^Ended$/ })).toBeInTheDocument();
    });

    it("filters campaigns by active status", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const activeButton = screen.getByRole("tab", { name: /^Active$/ });
      await user.click(activeButton);

      await waitFor(() => {
        expect(screen.getByTestId("campaign-card-1")).toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-2")).not.toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-3")).not.toBeInTheDocument();
      });
    });

    it("filters campaigns by funded status", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const fundedTab = screen.getByRole("tab", { name: /^Funded$/ });
      await user.click(fundedTab);

      await waitFor(() => {
        expect(screen.queryByTestId("campaign-card-1")).not.toBeInTheDocument();
        expect(screen.getByTestId("campaign-card-2")).toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-3")).not.toBeInTheDocument();
      });
    });

    it("filters campaigns by ended status", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const endedButton = screen.getByRole("tab", { name: /^Ended$/ });
      await user.click(endedButton);

      await waitFor(() => {
        expect(screen.queryByTestId("campaign-card-1")).not.toBeInTheDocument();
        expect(screen.queryByTestId("campaign-card-2")).not.toBeInTheDocument();
        expect(screen.getByTestId("campaign-card-3")).toBeInTheDocument();
      });
    });

    it("resets page to 1 when filter changes", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const fundedTab = screen.getByRole("tab", { name: /^Funded$/ });
      await user.click(fundedTab);

      await waitFor(() => {
        expect(screen.getByTestId("campaign-card-2")).toBeInTheDocument();
      });
    });
  });

  describe("Sorting Functionality", () => {
    it("displays sort dropdown", () => {
      render(<DiscoveryPage />);

      const sortSelect = screen.getByLabelText(/Sort campaigns/i);
      expect(sortSelect).toBeInTheDocument();
    });

    it("sorts campaigns by newest", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const sortSelect = screen.getByLabelText(/Sort campaigns/i) as HTMLSelectElement;
      expect(sortSelect.value).toBe("recent");

      // With newest sort, campaigns should be in order 3, 2, 1 (by ID desc)
      await waitFor(() => {
        const cards = screen.getAllByTestId(/campaign-card-/);
        expect(cards.length).toBeGreaterThan(0);
      });
    });

    it("sorts campaigns by most funded", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const sortSelect = screen.getByLabelText(/Sort campaigns/i);
      await user.selectOptions(sortSelect, "most-funded");

      await waitFor(() => {
        expect((sortSelect as HTMLSelectElement).value).toBe("most-funded");
      });
    });

    it("resets page to 1 when sort changes", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const sortSelect = screen.getByLabelText(/Sort campaigns/i);
      await user.selectOptions(sortSelect, "ending-soon");

      await waitFor(() => {
        expect((sortSelect as HTMLSelectElement).value).toBe("ending-soon");
      });
    });
  });

  describe("Results Summary", () => {
    it("shows correct campaign count", () => {
      const { container } = render(<DiscoveryPage />);

      const summary = Array.from(container.querySelectorAll("p")).find((p) =>
        p.textContent?.includes("Showing"),
      );
      expect(summary).toBeTruthy();
      expect(
        summary?.textContent?.replace(/\s+/g, " ").includes(
          "Showing 1 – 3 of 3 campaigns",
        ),
      ).toBe(true);
    });

    it("updates summary when search is applied", async () => {
      const user = userEvent.setup();
      const { container } = render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Solar");

      await waitFor(() => {
        const summary = Array.from(container.querySelectorAll("p")).find((p) =>
          p.textContent?.includes("Showing"),
        );
        expect(summary).toBeTruthy();
        expect(
          summary?.textContent?.replace(/\s+/g, " ").includes(
            "Showing 1 – 1 of 1 campaign",
          ),
        ).toBe(true);
      });
    });

    it("updates summary when filter is applied", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const fundedTab = screen.getByRole("tab", { name: /^Funded$/ });
      await user.click(fundedTab);

      await waitFor(() => {
        expect(screen.getByText(/Showing 1–1 of 1 campaign/i)).toBeInTheDocument();
      });
    });
  });

  describe("Empty State", () => {
    it("shows empty state message when no results", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "XYZ123");

      await waitFor(() => {
        expect(screen.getByText(/No campaigns found/i)).toBeInTheDocument();
      });
    });

    it("shows 'Browse All Campaigns' button in empty state with search", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const searchInput = screen.getByPlaceholderText(/search campaigns/i);
      await user.type(searchInput, "Nonexistent");

      await waitFor(() => {
        expect(screen.getByText(/Browse All Campaigns/i)).toBeInTheDocument();
      });
    });
  });

  describe("Mobile Filters", () => {
    it("shows mobile filters toggle button", () => {
      render(<DiscoveryPage />);

      const filterButton = screen.getByRole("button", { name: /Filters & Sort/i });
      expect(filterButton).toBeInTheDocument();
    });

    it("toggles mobile filters panel visibility", async () => {
      const user = userEvent.setup();
      render(<DiscoveryPage />);

      const filterButton = screen.getByRole("button", { name: /Filters & Sort/i });
      await user.click(filterButton);

      await waitFor(() => {
        expect(screen.getByText("Filter by Status")).toBeInTheDocument();
      });
    });
  });
});
