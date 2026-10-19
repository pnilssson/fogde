import { CaretRightIcon } from "@phosphor-icons/react";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "@/components/ui/button";
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
import { callOwn } from "@/lib/host";
import { machinesQuery } from "@/lib/machines";

type Place = "computer" | "server";

interface Props {
  onboarded: boolean;
  onCancel: () => void;
  onComputer: () => Promise<void>;
  onServer: () => Promise<void>;
}

export function ChooseWhere({
  onboarded,
  onCancel,
  onComputer,
  onServer,
}: Props) {
  const queries = useQueryClient();
  const [going, setGoing] = useState<Place>();

  const go = async (place: Place) => {
    setGoing(place);
    try {
      if (place === "server") {
        await onServer();
        return;
      }
      await callOwn("machines.addComputer");
      await queries.invalidateQueries(machinesQuery);
      await onComputer();
    } catch (error) {
      toast.add({
        type: "error",
        title: "Could not add this computer",
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
      <h1 className="type-display">
        {onboarded ? "Add a machine" : "Where should your agents run?"}
      </h1>
      <p className="type-body">
        {onboarded
          ? "Each machine keeps its own repos, sign-ins and agents."
          : "Start with one machine. You can add more anytime in Settings."}
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
      {onboarded && (
        <Button variant="ghost" size="sm" onClick={onCancel}>
          Back
        </Button>
      )}
    </section>
  );
}
