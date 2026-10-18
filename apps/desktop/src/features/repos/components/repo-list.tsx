import { DoneBadge } from "@/components/done-badge";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Field,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { repoKey } from "@/features/repos/lib/repo-key";
import type { RemoteRepo } from "@/generated/protocol";

export type Progress = "cloning" | "cloned";

interface Props {
  hosts: string[];
  repos: RemoteRepo[];
  progress: (repo: RemoteRepo) => Progress | undefined;
  onClone: (repo: RemoteRepo) => void;
}

export function RepoList({ hosts, repos, progress, onClone }: Props) {
  const groups = hosts
    .map((host) => ({
      host,
      repos: repos.filter((repo) => repo.host === host),
    }))
    .filter((group) => group.repos.length > 0);

  return (
    <ScrollArea className="h-64 w-full">
      <div className="flex flex-col gap-4 pr-4">
        {groups.length === 0 && (
          <p className="type-body">No repositories found</p>
        )}
        {groups.map((group) => (
          <FieldSet key={group.host}>
            {hosts.length > 1 && (
              <FieldLegend variant="label">{group.host}</FieldLegend>
            )}
            <FieldGroup data-slot="checkbox-group">
              {group.repos.map((repo) => {
                const key = repoKey(repo);
                const status = progress(repo);

                return (
                  <Field key={key} orientation="horizontal">
                    <Checkbox
                      id={key}
                      checked={status !== undefined}
                      disabled={status !== undefined}
                      onCheckedChange={(checked) => {
                        if (checked) {
                          onClone(repo);
                        }
                      }}
                    />
                    <FieldLabel htmlFor={key} className="min-w-0">
                      <span className="truncate">
                        {repo.owner}/{repo.name}
                      </span>
                    </FieldLabel>
                    {status === "cloning" && (
                      <Badge variant="secondary">
                        <Spinner aria-hidden />
                        Cloning
                      </Badge>
                    )}
                    {status === "cloned" && <DoneBadge>Cloned</DoneBadge>}
                  </Field>
                );
              })}
            </FieldGroup>
          </FieldSet>
        ))}
      </div>
    </ScrollArea>
  );
}
