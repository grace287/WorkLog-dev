import { PortfolioSchema, type PortfolioData } from "./schemas";

const API_URL =
  process.env.NEXT_PUBLIC_API_URL ?? "https://api.worklog.dev";

export async function getPortfolio(slug: string): Promise<PortfolioData | null> {
  const res = await fetch(`${API_URL}/api/v1/portfolios/${slug}`, {
    next: { revalidate: 60 }, // ISR: 60초마다 재검증
  });

  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`API error ${res.status}`);

  const json = await res.json();
  return PortfolioSchema.parse(json);
}

export async function getAllSlugs(): Promise<string[]> {
  // SSG용: 공개 포트폴리오 slug 목록 (API가 없으면 빈 배열 반환)
  try {
    const res = await fetch(`${API_URL}/api/v1/portfolios/public`, {
      next: { revalidate: 300 },
    });
    if (!res.ok) return [];
    const data = await res.json();
    return Array.isArray(data) ? data.map((p: { slug: string }) => p.slug) : [];
  } catch {
    return [];
  }
}
