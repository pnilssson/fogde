import { CaretRightIcon } from "@phosphor-icons/react";
import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { useState } from "react";

import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemTitle,
} from "@/components/ui/item";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import { serverQuery } from "@/features/server/api/server";
import { switchHost } from "@/features/server/lib/switch-host";
import { callOwn } from "@/lib/host";

type Place = "computer" | "server";

interface Props {
  onComputer: () => Promise<void>;
  onServer: () => Promise<void>;
}

export function ChooseWhere({ onComputer, onServer }: Props) {
  const { data: server } = useSuspenseQuery(serverQuery);
  const queries = useQueryClient();
  const [going, setGoing] = useState<Place>();

  const go = async (place: Place) => {
    setGoing(place);
    try {
      if (place === "server") {
        await onServer();
        return;
      }
      if (server !== null) {
        await callOwn("server.clear");
        switchHost(queries);
        queries.setQueryData(serverQuery.queryKey, null);
      }
      await onComputer();
    } catch (error) {
      toast.add({
        type: "error",
        title: "Could not switch to this computer",
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setGoing(undefined);
    }
  };

  const places = [
    {
      place: "computer",
      title: "This computer",
      description: "Agents work on the repos on this computer.",
    },
    {
      place: "server",
      title: "A server",
      description:
        "Agents work on a Linux server you reach with SSH and keep going when this computer sleeps.",
    },
  ] as const;

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <h1 className="type-display">Where should your agents run?</h1>
      <p className="type-body">
        Set up your first workspace. You can add more anytime in Settings.
      </p>
      <ItemGroup className="gap-2">
        {places.map(({ place, title, description }) => (
          <Item
            key={place}
            variant="outline"
            render={
              <button
                type="button"
                aria-busy={going === place}
                disabled={going !== undefined}
                onClick={() => void go(place)}
              />
            }
          >
            <ItemContent>
              <ItemTitle>{title}</ItemTitle>
              <ItemDescription>{description}</ItemDescription>
            </ItemContent>
            <ItemActions>
              {going === place ? <Spinner aria-hidden /> : <CaretRightIcon />}
            </ItemActions>
          </Item>
        ))}
      </ItemGroup>
    </section>
  );
}
