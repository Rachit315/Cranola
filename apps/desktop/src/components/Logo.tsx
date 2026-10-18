/** The Scribewell mark: lines of a note with the middle line drawn as a waveform. */
export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 1024 1024" className={className} aria-hidden focusable="false">
      <rect width="1024" height="1024" rx="232" className="fill-accent" />
      <g className="fill-on-accent">
        <rect x="232" y="276" width="560" height="64" rx="32" />
        <rect x="232" y="472" width="40" height="80" rx="20" />
        <rect x="318" y="432" width="40" height="160" rx="20" />
        <rect x="404" y="392" width="40" height="240" rx="20" />
        <rect x="490" y="448" width="40" height="128" rx="20" />
        <rect x="576" y="412" width="40" height="200" rx="20" />
        <rect x="662" y="464" width="40" height="96" rx="20" />
        <rect x="748" y="484" width="40" height="56" rx="20" />
        <rect x="232" y="684" width="380" height="64" rx="32" />
      </g>
    </svg>
  );
}
