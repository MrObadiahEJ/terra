interface Props {
  size?: number
  className?: string
}

export default function TerraLogo({ size = 34, className }: Props) {
  return (
    <svg
      className={className}
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      role="img"
      aria-label="Terra"
    >
      <defs>
        <linearGradient id="terra-orbit" x1="7" y1="38" x2="40" y2="7" gradientUnits="userSpaceOnUse">
          <stop stopColor="#41D6B3" />
          <stop offset="1" stopColor="#9AE65B" />
        </linearGradient>
        <linearGradient id="terra-land" x1="13" y1="31" x2="34" y2="17" gradientUnits="userSpaceOnUse">
          <stop stopColor="#77D77D" />
          <stop offset="1" stopColor="#C6EE69" />
        </linearGradient>
      </defs>
      <ellipse cx="24" cy="24" rx="19" ry="10" transform="rotate(-38 24 24)" stroke="url(#terra-orbit)" strokeWidth="2.2" />
      <path d="m13.1 23.3 8.5-5.1 12.8 2.2.9 9.8-10 6.3-11.4-4.1-.8-9.1Z" fill="url(#terra-land)" fillOpacity=".96" stroke="#F5FFE8" strokeOpacity=".8" strokeWidth="1.1" strokeLinejoin="round" />
      <path d="m13.3 23.6 10 3.2 11-6.1M23.3 26.8l1 9.4M18.7 20l.8 8.7 5.8 7.2" stroke="#276D5C" strokeOpacity=".68" strokeWidth="1" strokeLinejoin="round" />
      <path d="M28.5 15.6a4.1 4.1 0 0 1 5.8 0l2.2 2.2a4.1 4.1 0 0 1 0 5.8l-4 4a4.1 4.1 0 0 1-5.8 0" stroke="#E8F7D8" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M19.5 32.4a4.1 4.1 0 0 1-5.8 0l-1.3-1.3a4.1 4.1 0 0 1 0-5.8l4-4a4.1 4.1 0 0 1 5.8 0" stroke="#E8F7D8" strokeWidth="2.5" strokeLinecap="round" />
      <circle cx="39.1" cy="9.8" r="2.1" fill="#C6EE69" />
    </svg>
  )
}
