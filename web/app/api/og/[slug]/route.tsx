import { ImageResponse } from "next/og";
import { getPortfolio } from "@/lib/api";

export const runtime = "edge";

export async function GET(
  _req: Request,
  { params }: { params: Promise<{ slug: string }> }
) {
  const { slug } = await params;
  const portfolio = await getPortfolio(slug);

  const title = portfolio?.project_id ?? slug;
  const tasks = portfolio?.tasks.length ?? 0;
  const commits = portfolio?.tasks.reduce((s, t) => s + t.commits.length, 0) ?? 0;
  const verified = portfolio?.tasks.reduce(
    (s, t) => s + t.commits.filter((c) => c.verified).length, 0
  ) ?? 0;

  return new ImageResponse(
    (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          width: "100%",
          height: "100%",
          background: "#09090b",
          padding: 64,
          fontFamily: "sans-serif",
        }}
      >
        <div style={{ color: "#71717a", fontSize: 18, marginBottom: 12 }}>
          worklog.dev/p/{slug}
        </div>
        <div style={{ color: "#fafafa", fontSize: 56, fontWeight: 700, lineHeight: 1.1 }}>
          {title}
        </div>
        <div style={{ display: "flex", gap: 32, marginTop: 32, color: "#a1a1aa", fontSize: 22 }}>
          <span>{tasks} tasks</span>
          <span>{commits} commits</span>
          <span style={{ color: "#4ade80" }}>{verified} verified</span>
        </div>
      </div>
    ),
    { width: 1200, height: 630 }
  );
}
