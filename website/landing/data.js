// Sample build orders + settings, lifted from the upstream repo
// (build-orders/*.yaml). Used by the embedded overlay demo.

window.BUILD_ORDERS = [
  {
    id: "scouts-generic",
    name: "Scouts (Generic)",
    civilization: "Generic",
    difficulty: "Beginner",
    description: "Standard 21+2 pop scout rush. Works for most cavalry civs.",
    tags: ["scouts", "feudal-aggression", "beginner-friendly"],
    steps: [
      { action: "6 vills on sheep, 1 vill builds house then to sheep", villagers_assigned: { food: 7 } },
      { action: "Next 3 vills to sheep under TC", villagers_assigned: { food: 10 } },
      { action: "4 vills to wood (build lumber camp)", notes: "Send to the closest woodline. Build a lumber camp.", villagers_assigned: { food: 10, wood: 4 } },
      { action: "Lure first boar", notes: "Shoot boar once with a villager, then garrison in TC. Don't let boar rot." },
      { action: "Build mill on berries, 3 vills to berries", villagers_assigned: { food: 11, wood: 4 } },
      { action: "Lure second boar" },
      { action: "2 more vills to wood", villagers_assigned: { food: 11, wood: 6 } },
      { action: "Build second house, 2 vills to farms" },
      { action: "Click up to Feudal Age", notes: "While advancing: send 2 sheep vills to wood." },
      { action: "Build barracks while advancing", notes: "Use a villager that finished a farm." },
      { action: "Build stable immediately in Feudal", notes: "Start producing scouts. Keep making farms." }
    ]
  },
  {
    id: "archers-britons",
    name: "Archers (Britons)",
    civilization: "Britons",
    difficulty: "Beginner",
    description: "22 pop flush into archers. Britons get +1 range in Castle Age.",
    tags: ["archers", "feudal-aggression", "britons"],
    steps: [
      { action: "6 vills on sheep", villagers_assigned: { food: 6 } },
      { action: "4 vills to wood (lumber camp)", villagers_assigned: { food: 6, wood: 4 } },
      { action: "Lure boar, next vill to berries (build mill)" },
      { action: "3 more vills to berries", villagers_assigned: { food: 10, wood: 4 } },
      { action: "Lure second boar, 3 vills to gold (mining camp)", villagers_assigned: { food: 10, wood: 4, gold: 3 } },
      { action: "2 more vills to wood, build house", villagers_assigned: { food: 10, wood: 6, gold: 3 } },
      { action: "2 vills to farms", villagers_assigned: { food: 12, wood: 6, gold: 3 } },
      { action: "Click up to Feudal Age", notes: "Build barracks with one of the wood villagers while advancing." },
      { action: "Build 2 archery ranges in Feudal", notes: "Non-stop archer production. Research fletching." },
      { action: "Send archers across the map", notes: "Target enemy woodlines and gold miners." }
    ]
  },
  {
    id: "fast-castle-cavalier",
    name: "Fast Castle into Knights",
    civilization: "Generic",
    difficulty: "Intermediate",
    description: "27+2 pop fast castle into knight production. Works for Franks, Berbers, Huns.",
    tags: ["fast-castle", "knights", "castle-age"],
    steps: [
      { action: "6 vills on sheep", villagers_assigned: { food: 6 } },
      { action: "4 vills to wood (lumber camp)" },
      { action: "Lure first boar" },
      { action: "Build mill on berries, 2 to berries" },
      { action: "Lure second boar, 2 more to berries", villagers_assigned: { food: 11, wood: 4 } },
      { action: "3 more to wood (second lumber camp if needed)" },
      { action: "5 to farms", notes: "Reseed farms as sheep and boar run out." },
      { action: "3 to gold (mining camp)" },
      { action: "Click up to Feudal Age", notes: "Build barracks while advancing." },
      { action: "Click up to Castle Age immediately", notes: "Build blacksmith + stable while advancing to Feudal." },
      { action: "Build 2 stables, start knight production", notes: "Research bloodlines and husbandry when affordable." },
      { action: "Boom behind knights — add TCs", notes: "Build 2 additional TCs. Keep producing knights and villagers." }
    ]
  }
];

window.HOTKEYS = {
  advance_step: "Ctrl+Alt+Right",
  previous_step: "Ctrl+Alt+Left",
  reset: "Ctrl+Alt+R",
  pause_capture: "Ctrl+Alt+P",
  toggle_visibility: "Ctrl+Alt+H",
  toggle_click_through: "Ctrl+Alt+C"
};
