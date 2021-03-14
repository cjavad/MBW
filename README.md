# MBW
KMG (Københavns Mediegymnasium) Game Jam Entry 2021

# Map / Gameplay

## Dag/Nat cyklus

- Tickrate per sekund in-game
    * 20 ticks per sekund
- En dag er 4,8 irl minutter
    * Hvert rigtig sekund skal der gå 5 in-game minutter
- Et tick er 15 sekunder in-game

## Overordnet typer "felter"

- Vej
- Barrikade
- Bolig felt
    * Beboer antal
    * Sammensatte
    * Enkel indgang
    * Kan invitere flere end beboer antal til fest eller lign.
- Job felter
    * Arbejdspladser
    * Sammensatte
    * Enkel indgang
- Supermarkeder / butikker
    * Sammensatte
    * Enkel indgang
- Test centre
    * Enkelt felter
- Læge klinik
    * Venteliste
- Offentlig toileter
    * Ew

## Overordnet metadata NPC

- Alder
    * Vægtede dødsrate
    * Vægtede smitterate
- Sex
    * Vægtede smitterate
- Arbejde
    * titel / arbejdsplads
    * Arbejdstimer
    * Vægtede smitterate
    * Vægtede kontakttal
- Vaner
    * Mundbind
    * Hygiejne (for ofte vasker de hænder)
    * Bekendte (vennekreds / omgangskreds) | Kontakttal

## Spiller interaktion

2 overordnet typer af interaktion:

1. Tile based
    - Placere tiles
2. AoE based
    - Giver en effekt (fra en tile)

### Player 1: virus ability idéer

Party Impulses: lav en fest der trækker tilfældige npcer hen til et område

Antivax campaign: folk der går forbi et bestemt område vil nægte at blive vaccineret

Roadblock: Blokere paths

Social impulses: mennesker samles i dette område efter arbejde

Økonomisk crash: Folk går ikke på arbejde

### Player 2: læge something player

Testcenter: Npcer der går forbi et område vil blive testet, positive tilfælde vil blive hjemme i noget tid.

Nedlukning: stop procentdel af npcer fra at bevæge sig ud fra deres hjem

Lokal nedlukning: Bloker npcer fra at gå igennem et område

Vaccinecenter: Gør npcer immune overfor infektion, har lav radius/duration

Maskekampagne: Får folk i et område til bruge masker i noget tid.

## Tortillia Protokolen

> **Ingen Salsa/Tacos**

- Initial game creation
    * Map generation / distribution
    * Player role selection (by server)
