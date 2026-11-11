import type { Conversation } from "../domain/conversation";

/**
 * Fixed sample conversation for reviewing the layout in demo mode.
 * It is never produced by audio capture or by a provider, and the UI must
 * always label it as demo data.
 */
export const DEMO_CONVERSATION: Conversation = {
  segments: [
    {
      id: "demo-1",
      source: "system",
      startMs: 4_000,
      text: "Hi, thanks for joining. We're looking for someone to rebuild our online store.",
      language: "en",
      isFinal: true,
    },
    {
      id: "demo-2",
      source: "system",
      startMs: 11_000,
      text: "How long would a project like that usually take, and what would it cost?",
      language: "en",
      isFinal: true,
    },
    {
      id: "demo-3",
      source: "microphone",
      startMs: 19_000,
      text: "Thanks for the details. Could you tell me how many products the store has today?",
      language: "en",
      isFinal: true,
    },
    {
      id: "demo-4",
      source: "system",
      startMs: 26_000,
      text: "We have around four hundred products, and we'd also like",
      language: "en",
      isFinal: false,
    },
  ],
  translations: {
    "demo-1": {
      status: "done",
      targetLanguage: "az",
      text: "Salam, qoşulduğunuz üçün təşəkkür edirik. Onlayn mağazamızı yenidən quracaq birini axtarırıq.",
      unpreserved: [],
    },
    "demo-2": {
      status: "done",
      targetLanguage: "az",
      text: "Belə bir layihə adətən nə qədər vaxt aparır və neçəyə başa gəlir?",
      unpreserved: [],
    },
    "demo-3": { status: "pending" },
  },
  replies: {
    status: "ready",
    failure: null,
    generation: 1,
    suggestions: [
      {
        style: "professional",
        text: "Thank you. To give you an accurate timeline and quote, could you share which features the new store must have at launch?",
        meaning:
          "Təşəkkür edirəm. Dəqiq müddət və qiymət təklifi verə bilməyim üçün yeni mağazada ilk gündən hansı funksiyaların olmalı olduğunu bildirə bilərsiniz?",
        asksForMissingFacts: true,
      },
      {
        style: "friendly",
        text: "That sounds like a great project! Around four hundred products is very manageable. What matters most to you in the new store?",
        meaning:
          "Bu, əla layihəyə oxşayır! Dörd yüzə yaxın məhsul tam idarəolunandır. Yeni mağazada sizin üçün ən vacib olan nədir?",
        asksForMissingFacts: false,
      },
      {
        style: "concise",
        text: "Got it. Which payment and delivery options do you need?",
        meaning: "Aydındır. Hansı ödəniş və çatdırılma seçimləri lazımdır?",
        asksForMissingFacts: false,
      },
    ],
  },
};
