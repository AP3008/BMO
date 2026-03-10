import type { QuickPanel } from "../../store";
import { FaceCard } from "./FaceCard";
import { ChatCard } from "./ChatCard";
import { TimerCard } from "./TimerCard";
import { CalendarCard } from "./CalendarCard";
import { NotesCard } from "./NotesCard";
import { SettingsCard } from "./SettingsCard";

const CARDS: Record<QuickPanel, React.FC> = {
  face: FaceCard,
  chat: ChatCard,
  timer: TimerCard,
  calendar: CalendarCard,
  notes: NotesCard,
  settings: SettingsCard,
};

export function QuickCardContent({ panel }: { panel: QuickPanel | null }) {
  if (!panel) return null;
  const Card = CARDS[panel];
  return <Card />;
}
