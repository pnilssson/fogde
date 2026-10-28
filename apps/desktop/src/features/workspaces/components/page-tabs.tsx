import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { showTiles } from "@/features/workspaces/lib/tiles";
import { openPage, type Page, usePage } from "@/lib/page";

type Tab = Page | "workspace";

const tabs: { tab: Tab; label: string }[] = [
  { tab: "workspace", label: "Workspace" },
  { tab: "notes", label: "Notes" },
  { tab: "monitor", label: "Monitor" },
];

export function PageTabs({ machine }: { machine: string | undefined }) {
  const page = usePage();

  return (
    <Tabs
      value={page ?? "workspace"}
      className="gap-0"
      onValueChange={(next: Tab) => {
        if (next === "workspace") {
          showTiles();
          return;
        }

        openPage(next, machine);
      }}
    >
      <TabsList aria-label="Sections">
        {tabs.map(({ tab, label }) => (
          <TabsTrigger key={tab} value={tab}>
            {label}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
