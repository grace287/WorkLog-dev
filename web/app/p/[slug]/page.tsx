import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { getPortfolio, getAllSlugs } from "@/lib/api";
import { PortfolioView } from "@/components/PortfolioView";

interface Props {
  params: Promise<{ slug: string }>;
}

// ISR: 60초마다 재검증
export const revalidate = 60;

export async function generateStaticParams() {
  const slugs = await getAllSlugs();
  return slugs.map((slug) => ({ slug }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const portfolio = await getPortfolio(slug);
  if (!portfolio) return { title: "Not Found" };

  return {
    title: portfolio.project_id,
    description: `${portfolio.tasks.length} tasks · ${portfolio.tasks.reduce((s, t) => s + t.commits.length, 0)} commits — worklog.dev/p/${slug}`,
    openGraph: {
      title: `${portfolio.project_id} — worklog`,
      description: `Verified Git portfolio by worklog`,
      url: `https://worklog.dev/p/${slug}`,
      images: [`/api/og/${slug}`],
    },
  };
}

export default async function PortfolioPage({ params }: Props) {
  const { slug } = await params;
  const portfolio = await getPortfolio(slug);
  if (!portfolio) notFound();
  return <PortfolioView portfolio={portfolio} />;
}
