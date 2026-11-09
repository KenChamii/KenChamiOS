import { openUrl } from "@tauri-apps/plugin-opener";

const CREDITS = [
  { name: "AniList", url: "https://anilist.co", text: "anime and manga data" },
  { name: "RAWG", url: "https://rawg.io", text: "game data" },
  { name: "TMDB", url: "https://www.themoviedb.org", text: "movie and show data" },
  { name: "MusicBrainz", url: "https://musicbrainz.org", text: "music data" },
  { name: "Cover Art Archive", url: "https://coverartarchive.org", text: "album covers" },
];

export function SourceCredits() {
  return (
    <div className="mt-4 flex flex-col gap-2 text-xs text-muted-foreground">
      <p>
        This product uses the TMDB API but is not endorsed, certified, or otherwise approved by
        TMDB.
      </p>
      <p>
        Media data from{" "}
        {CREDITS.map((credit, index) => (
          <span key={credit.name}>
            {index > 0 && ", "}
            <button type="button" className="underline" onClick={() => openUrl(credit.url)}>
              {credit.name}
            </button>{" "}
            ({credit.text})
          </span>
        ))}
        .
      </p>
    </div>
  );
}
