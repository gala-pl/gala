export const siteConfig = {
  name: "Gala",
  description: "Documentation for the Gala programming language.",
  url: process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000",
  repositoryUrl: process.env.NEXT_PUBLIC_REPOSITORY_URL,
  socialUrl: process.env.NEXT_PUBLIC_SOCIAL_URL,
  author: "Gala team",
} as const;

export function absoluteUrl(path: string): string {
  return new URL(path, siteConfig.url).toString();
}
