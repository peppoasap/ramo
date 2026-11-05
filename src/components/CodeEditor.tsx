import { useEffect, useRef } from "react";
import type { LineRange } from "../conflictNavigation";
import { basicSetup } from "codemirror";
import { Compartment, EditorState } from "@codemirror/state";
import {
  Decoration,
  GutterMarker,
  gutter,
  lineNumbers,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { LanguageDescription } from "@codemirror/language";
import { oneDark } from "@codemirror/theme-one-dark";
import { languages } from "@codemirror/language-data";
function markers(view: EditorView): DecorationSet {
  const ranges = [];
  for (let i = 1; i <= view.state.doc.lines; i++) {
    const line = view.state.doc.line(i);
    if (/^(<<<<<<<|=======|>>>>>>>|\|\|\|\|\|\|\|)/.test(line.text))
      ranges.push(
        Decoration.line({ class: "cm-conflict-marker" }).range(line.from),
      );
  }
  return Decoration.set(ranges);
}
const conflictHighlights = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = markers(view);
    }
    update(update: ViewUpdate) {
      if (update.docChanged) this.decorations = markers(update.view);
    }
  },
  { decorations: (value) => value.decorations },
);
export function CodeEditor({
  value,
  onChange,
  editable = false,
  label,
  filename,
  activeRange,
  navigation = 0,
  follow = false,
  selectedLines,
  onPickLine,
  onSelectionChange,
}: {
  value: string;
  onChange?: (value: string) => void;
  editable?: boolean;
  label: string;
  filename: string;
  activeRange?: LineRange | null;
  navigation?: number;
  follow?: boolean;
  selectedLines?: readonly number[];
  onPickLine?: (line: number) => void;
  onSelectionChange?: (range: { from: number; to: number }) => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const editor = useRef<EditorView | null>(null);
  const selectionGutter = useRef(new Compartment());
  const activeHighlight = useRef(new Compartment());
  const previousNavigation = useRef(navigation);
  const pick = useRef(onPickLine);
  pick.current = onPickLine;
  const selectionCallback = useRef(onSelectionChange);
  selectionCallback.current = onSelectionChange;
  const callback = useRef(onChange);
  callback.current = onChange;
  useEffect(() => {
    const language = new Compartment();
    const view = new EditorView({
      parent: host.current!,
      state: EditorState.create({
        doc: value,
        extensions: [
          basicSetup,
          selectionGutter.current.of([]),
          activeHighlight.current.of([]),
          oneDark,
          EditorState.lineSeparator.of(value.includes("\r\n") ? "\r\n" : "\n"),
          language.of([]),
          conflictHighlights,
          EditorState.readOnly.of(!editable),
          EditorView.editable.of(editable),
          EditorView.contentAttributes.of({
            "aria-label": label,
            spellcheck: "false",
          }),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) callback.current?.(update.state.sliceDoc());
            if (update.selectionSet || update.docChanged) {
              const range = update.state.selection.main;
              selectionCallback.current?.({
                from: update.state.sliceDoc(0, range.from).length,
                to: update.state.sliceDoc(0, range.to).length,
              });
            }
          }),
          EditorView.theme(
            {
              "&": {
                height: "100%",
                backgroundColor: "#171f26",
                color: "#d5dfe7",
              },
              ".cm-scroller": {
                overflow: "auto",
                fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
                fontSize: "12px",
                lineHeight: "1.7",
              },
              ".cm-content": { padding: "8px 0", caretColor: "#b1efd0" },
              ".cm-gutters": {
                backgroundColor: "#171f26",
                color: "#657681",
                borderRight: "1px solid #2b3741",
              },
              ".cm-activeLine, .cm-activeLineGutter": {
                backgroundColor: "#ffffff07",
              },
              "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection":
                { backgroundColor: "#375a6e !important" },
              ".cm-cursor": { borderLeftColor: "#b1efd0" },
              ".cm-conflict-marker": {
                backgroundColor: "#ac79402c",
                color: "#edc383",
              },
              ".cm-panels": { backgroundColor: "#25313a", color: "#d5dfe7" },
              ".cm-searchMatch": { backgroundColor: "#5a724d" },
            },
            { dark: true },
          ),
        ],
      }),
    });
    editor.current = view;
    const syntax = LanguageDescription.matchFilename(languages, filename);
    let active = true;
    if (syntax)
      void syntax.load().then((support) => {
        if (active) view.dispatch({ effects: language.reconfigure(support) });
      });
    return () => {
      active = false;
      editor.current = null;
      view.destroy();
    };
  }, [editable, label, filename]);
  useEffect(() => {
    const view = editor.current;
    if (view && view.state.sliceDoc() !== value)
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value },
      });
  }, [value]);
  useEffect(() => {
    const view = editor.current;
    if (!view) return;
    if (!onPickLine) {
      view.dispatch({ effects: selectionGutter.current.reconfigure([]) });
      return;
    }
    const chosen = new Set(selectedLines || []);
    class PickLine extends GutterMarker {
      constructor(
        readonly number: number,
        readonly checked: boolean,
      ) {
        super();
      }
      eq(other: PickLine) {
        return this.number === other.number && this.checked === other.checked;
      }
      toDOM() {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "cm-line-pick";
        button.setAttribute(
          "aria-label",
          `Include line ${this.number} from ${label}`,
        );
        button.setAttribute("aria-pressed", String(this.checked));
        button.title = `${this.checked ? "Deselect" : "Select"} line ${this.number}`;
        button.textContent = this.checked ? "✓" : "+";
        button.addEventListener("mousedown", (event) => {
          event.preventDefault();
          event.stopPropagation();
        });
        button.addEventListener("click", (event) => {
          event.preventDefault();
          event.stopPropagation();
          pick.current?.(this.number);
        });
        return button;
      }
    }
    const highlights = Array.from(chosen)
      .filter((number) => number > 0 && number <= view.state.doc.lines)
      .sort((a, b) => a - b)
      .map((number) =>
        Decoration.line({ class: "cm-picked-source-line" }).range(
          view.state.doc.line(number).from,
        ),
      );
    view.dispatch({
      effects: selectionGutter.current.reconfigure([
        gutter({
          class: "cm-pick-gutter",
          lineMarker: (view, line) => {
            const number = view.state.doc.lineAt(line.from).number;
            return new PickLine(number, chosen.has(number));
          },
        }),
        lineNumbers({
          domEventHandlers: {
            mousedown: (view, line, event) => {
              if (!(event instanceof MouseEvent) || event.button !== 0)
                return false;
              event.preventDefault();
              pick.current?.(view.state.doc.lineAt(line.from).number);
              return true;
            },
          },
        }),
        EditorView.editorAttributes.of({ "data-line-picking": "true" }),
        EditorView.decorations.of(Decoration.set(highlights)),
      ]),
    });
    // CodeMirror hides decorative gutters by default; these contain real controls.
    const column = view.dom.querySelector<HTMLElement>(".cm-pick-gutter");
    const gutters = column?.parentElement;
    gutters?.removeAttribute("aria-hidden");
    column?.setAttribute("role", "group");
    column?.setAttribute("aria-label", `Select lines from ${label}`);
    if (gutters)
      for (const sibling of Array.from(gutters.children)) {
        if (sibling !== column) sibling.setAttribute("aria-hidden", "true");
      }
  }, [selectedLines, !!onPickLine, editable, label, filename]);
  useEffect(() => {
    const view = editor.current;
    if (!view) return;
    const ranges = [];
    if (activeRange) {
      for (
        let line = Math.max(1, activeRange.from);
        line <= Math.min(view.state.doc.lines, activeRange.to);
        line++
      )
        ranges.push(
          Decoration.line({ class: "cm-active-conflict" }).range(
            view.state.doc.line(line).from,
          ),
        );
    }
    view.dispatch({
      effects: activeHighlight.current.reconfigure(
        EditorView.decorations.of(Decoration.set(ranges)),
      ),
    });
  }, [activeRange?.from, activeRange?.to, value, editable, label, filename]);
  useEffect(() => {
    const view = editor.current;
    const requested = previousNavigation.current !== navigation;
    previousNavigation.current = navigation;
    if (!view || (!follow && !requested) || !activeRange) return;
    const line = view.state.doc.line(
      Math.max(1, Math.min(view.state.doc.lines, activeRange.from)),
    );
    // Navigation must not move the caret or replace a user's selected text.
    view.dispatch({
      effects: EditorView.scrollIntoView(line.from, { y: "center" }),
    });
  }, [navigation, follow, activeRange?.from, editable, label, filename]);
  return <div className="code-editor" ref={host} />;
}
