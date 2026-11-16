# Reader interface

## Design plan

- **Archetype:** native document window with an optional workspace source list and document inspector.
- **Information architecture:** Home lists recent documents, reading sessions, favorites and workspaces. Reader tabs select documents. Inspector segments select pages, outline, search, bookmarks and annotations.
- **Command map:** existing File, Edit, Tools and View commands and shortcuts remain available; panels open from toolbar or View, search from Command-F, quick open from Command-P.
- **Empty states:** a plain system symbol, a short title and concrete next action. No decorative medallions, hero gradients or colorful metadata capsules.
- **Feeling:** calm and predictable. Document content receives attention; navigation recedes.

## Appearance

Use AppKit materials for navigation and system label/separator colors for chrome. Icons are monochrome. Selected tabs use a neutral control background and semibold title. Lists use a restrained system accent selection that follows window emphasis. Thumbnail selection uses one flat system accent. Metadata is text, with red reserved for errors. Preserve the user's PDF and annotation colors. Honor light/dark appearance and system accessibility settings.

## Window behavior

Opening documents, changing Home/Reader mode, opening panels and restoring focus mode retain the chosen outer frame. Expanding a panel refits the PDF to the remaining viewport and retains the page. Manual zoom and manual window movement remain available. Automatic initial/restored placement stays inside the current display's visible frame, including displays smaller than the preferred minimum.

Tab titles live inside a clipped viewport and use an overflow menu when space runs out. The selected tab, overflow control and new-tab button remain visible. On displays too narrow for both panels plus the reader, opening one panel closes the opposite panel. Moving to a smaller display or losing a display sanitizes automatic placement.

## Verification

Run `swift test --scratch-path /tmp/pdf-reader-test` and `./scripts/verify.sh`. `scripts/visual_qa.sh` builds a separate QA bundle, creates synthetic PDFs and uses an isolated workspace index. Its window matrix checks frame equality, screen containment, PDF fitting and current-page retention in both appearances; captures and measurements stay under `/tmp`.
