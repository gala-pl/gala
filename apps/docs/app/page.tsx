import Link from 'next/link';
import { Button } from '@/components/ui/button';

export default function HomePage() {
  return (
    <div className="flex flex-col min-h-screen">
      <header className="border-b">
        <div className="container mx-auto flex h-14 items-center px-4">
          <Link href="/" className="font-bold text-lg mr-auto">
            Gala
          </Link>
          <Link
            href="/docs"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Docs
          </Link>
        </div>
      </header>

      <main className="flex-1 flex flex-col items-center justify-center px-4">
        <section className="max-w-2xl mx-auto text-center py-24">
          <h1 className="text-5xl font-bold tracking-tight mb-4">Gala</h1>
          <p className="text-xl text-muted-foreground mb-2">
            The hybrid quantum-classical programming language
          </p>
          <p className="text-sm text-muted-foreground/70 mb-8 max-w-lg mx-auto">
            A classical language whose compiler enforces the laws of quantum mechanics — no-cloning,
            reversibility, and safe uncomputation — while making hybrid quantum-classical machine
            learning ergonomic end to end.
          </p>
          <div className="flex items-center justify-center gap-4">
            <Button asChild>
              <Link href="/docs/getting-started/installation">Get Started</Link>
            </Button>
            <Button variant="outline" asChild>
              <Link href="/docs">Read Docs</Link>
            </Button>
          </div>
        </section>
      </main>
    </div>
  );
}
