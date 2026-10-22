import { useState } from 'react';

import type { MagiState } from '../hooks/useMagiSession';
import { PERSONAS } from '../lib/labels';
import { debateFor, openingFor, voteFor } from '../lib/sessionSelectors';
import UnitColumn from './UnitColumn';
import VerdictPanel from './VerdictPanel';

export default function ExtendedAnalysis({ state }: { state: MagiState }) {
  const [open, setOpen] = useState(false);
  const phaseLabel = state.phase.toUpperCase().replace('_', ' ');

  return (
    <>
      <section className="relative z-20 border-t-2 border-term-amber bg-black">
        <header className="flex flex-wrap items-center gap-2 px-3 py-1 text-[9px] tracking-[0.16em] text-term-dim">
          <span className="text-term-amber">EXTENDED ANALYSIS</span>
          <span>OPENING / DEBATE / FINAL VOTE</span>
          <span className="ml-auto">{phaseLabel}</span>
          <button
            type="button"
            className="border border-term-amber px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-amber hover:bg-term-amber hover:text-black"
            onClick={() => setOpen(true)}
          >
            EXPAND
          </button>
        </header>
      </section>

      {open ? (
        <>
          <div
            className="fixed inset-0 z-30 bg-black/70"
            onClick={() => setOpen(false)}
            role="presentation"
          />
          <div className="fixed inset-x-0 bottom-0 z-40 flex max-h-[62vh] flex-col border-t-2 border-term-amber bg-black">
            <header className="flex flex-wrap items-center gap-2 border-b-2 border-term-amber px-3 py-1.5 text-[9px] tracking-[0.16em] text-term-dim">
              <span className="text-term-amber">
                EXTENDED ANALYSIS // PROTOCOL
              </span>
              <span>OPENING / DEBATE / FINAL VOTE</span>
              <span className="ml-auto">{phaseLabel}</span>
              <button
                type="button"
                className="border border-term-amber px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-amber hover:bg-term-amber hover:text-black"
                onClick={() => setOpen(false)}
              >
                CLOSE
              </button>
            </header>

            <div className="min-h-0 flex-1 overflow-y-auto">
              <div className="grid grid-cols-3 border-b border-term-grid max-[900px]:grid-cols-1">
                {PERSONAS.map((persona, index) => (
                  <UnitColumn
                    key={persona}
                    index={index}
                    persona={persona}
                    status={state.nodes[persona]}
                    stream={state.streaming[persona]}
                    opening={openingFor(state.openings, persona)}
                    debate={debateFor(state.debate, persona)}
                    vote={voteFor(state.votes, persona)}
                    error={state.nodeErrors[persona]}
                    phase={state.phase}
                    readOnly={state.readOnly}
                  />
                ))}
              </div>
              <VerdictPanel votes={state.votes} phase={state.phase} />
            </div>
          </div>
        </>
      ) : null}
    </>
  );
}
