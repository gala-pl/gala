import { siteConfig } from "@/lib/site";
import { ImageResponse } from "next/og";

export const runtime = "edge";

export function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const title = searchParams.get("title") || siteConfig.name;
  const description = title.length > 60 ? null : searchParams.get("description") || siteConfig.description;

  return new ImageResponse(
    <div
      style={{
        background: "#0f1117",
        color: "#ffffff",
        display: "flex",
        flexDirection: "column",
        height: "100%",
        justifyContent: "space-between",
        padding: "64px",
        width: "100%",
      }}
    >
      <div style={{ color: "#9ca3af", display: "flex", fontSize: 28, fontWeight: 600 }}>{siteConfig.name}</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
        <div style={{ display: "flex", fontSize: 76, fontWeight: 700, letterSpacing: "-0.05em" }}>{title}</div>
        {description ? <div style={{ color: "#d1d5db", display: "flex", fontSize: 28 }}>{description}</div> : null}
      </div>
      <div style={{ borderTop: "1px solid #374151", color: "#9ca3af", display: "flex", fontSize: 24, paddingTop: 28 }}>
        Documentation for the Gala programming language
      </div>
    </div>,
    { width: 1200, height: 630 }
  );
}
