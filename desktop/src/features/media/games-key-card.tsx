import { useClearGamesKey, useGamesKeyStatus, useSetGamesKey } from "@/hooks/use-media";
import { ApiKeyCard } from "./api-key-card";

export function GamesKeyCard() {
  return (
    <ApiKeyCard
      title="Games search"
      intro="Game search uses RAWG and needs a free API key."
      helpUrl="https://rawg.io/apidocs"
      label="RAWG API key"
      status={useGamesKeyStatus()}
      save={useSetGamesKey()}
      clear={useClearGamesKey()}
    />
  );
}
