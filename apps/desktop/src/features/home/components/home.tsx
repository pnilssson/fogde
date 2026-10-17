import type { User } from "@/generated/protocol";

interface Props {
  user: User;
}

export function Home({ user }: Props) {
  return (
    <section className="flex max-w-sm flex-col items-start gap-2">
      <h1 className="type-display">Welcome, {user.name ?? user.login}</h1>
      <p className="type-body">Signed in to GitHub as {user.login}.</p>
    </section>
  );
}
