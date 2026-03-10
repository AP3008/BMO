import idle1 from "../../assets/BMO-face/BMO-Idle.svg";
import idle2 from "../../assets/BMO-face/BMO-Idle-2.svg";
import idle3 from "../../assets/BMO-face/BMO-Idle-3.svg";
import idle4 from "../../assets/BMO-face/BMO-Idle-4.svg";
import happy2 from "../../assets/BMO-face/BMO-Happy-2.svg";
import happy3 from "../../assets/BMO-face/BMO-Happy-3.svg";
import angry1 from "../../assets/BMO-face/BMO-Angry-1.svg";
import angry2 from "../../assets/BMO-face/BMO-Angry-2.svg";
import awe1 from "../../assets/BMO-face/BMO-Awe.svg";
import awe2 from "../../assets/BMO-face/BMO-Awe-2.svg";
import cheeky1 from "../../assets/BMO-face/BMO-Cheeky.svg";
import cheeky2 from "../../assets/BMO-face/BMO-Cheeky-2.svg";
import love1 from "../../assets/BMO-face/BMO-Love.svg";
import sad1 from "../../assets/BMO-face/BMO-Sad-1.svg";
import sad2 from "../../assets/BMO-face/BMO-Sad-2.svg";
import scared1 from "../../assets/BMO-face/BMO-Scared.svg";
import shocked1 from "../../assets/BMO-face/BMO-Shocked.svg";
import suspicious1 from "../../assets/BMO-face/BMO-Suspicious.svg";
import thinking1 from "../../assets/BMO-face/BMO-Thinking.svg";

export const DEFAULT_IDLE = idle1;

export const FACES = {
  happy: [happy2, happy3],
  idle: [idle1, idle2, idle3, idle4],
  angry: [angry1, angry2],
  awe: [awe1, awe2],
  cheeky: [cheeky1, cheeky2],
  love: [love1],
  sad: [sad1, sad2],
  scared: [scared1],
  shocked: [shocked1],
  suspicious: [suspicious1],
  thinking: [thinking1],
} as const;

export type FaceGroup = keyof typeof FACES;
