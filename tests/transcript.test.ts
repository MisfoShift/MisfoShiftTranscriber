import { describe, expect, it } from "vitest";
import { appendTranscript, transcriptFileName } from "../src/lib/transcript";

describe("appendTranscript", () => {
  it("adds confirmed segments on separate lines", () => {
    expect(appendTranscript("最初の文", "次の文")).toBe("最初の文\n次の文");
  });

  it("ignores an empty recognition result", () => {
    expect(appendTranscript("既存", "  ")).toBe("既存");
  });
});

describe("transcriptFileName", () => {
  it("creates a filesystem-friendly timestamp", () => {
    expect(transcriptFileName(new Date(2026, 7, 27, 9, 5))).toBe("transcript-20260827-0905.txt");
  });
});
