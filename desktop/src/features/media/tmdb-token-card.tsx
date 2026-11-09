import { useClearTmdbToken, useSetTmdbToken, useTmdbTokenStatus } from "@/hooks/use-media";
import { ApiKeyCard } from "./api-key-card";

export function TmdbTokenCard() {
  return (
    <ApiKeyCard
      title="Movies and shows search"
      intro="Movie and show search uses TMDB and needs a free API Read Access Token."
      helpUrl="https://www.themoviedb.org/settings/api"
      label="TMDB read access token"
      status={useTmdbTokenStatus()}
      save={useSetTmdbToken()}
      clear={useClearTmdbToken()}
    />
  );
}
