import Link from "next/link";

function GitHubIcon(props: any) {
  return (
    <svg viewBox="0 0 20 20" aria-hidden="true" {...props}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M10 1.667c-4.605 0-8.334 3.823-8.334 8.544 0 3.78 2.385 6.974 5.698 8.106.417.075.573-.182.573-.406 0-.203-.011-.875-.011-1.592-2.093.397-2.635-.522-2.802-1.002-.094-.246-.5-1.005-.854-1.207-.291-.16-.708-.556-.01-.567.656-.01 1.124.62 1.281.876.75 1.292 1.948.93 2.427.705.073-.555.291-.93.531-1.143-1.854-.213-3.791-.95-3.791-4.218 0-.929.322-1.698.854-2.296-.083-.214-.375-1.09.083-2.265 0 0 .698-.224 2.292.876a7.576 7.576 0 0 1 2.083-.288c.709 0 1.417.096 2.084.288 1.593-1.11 2.291-.875 2.291-.875.459 1.174.167 2.05.084 2.263.53.599.854 1.357.854 2.297 0 3.278-1.948 4.005-3.802 4.219.302.266.563.78.563 1.58 0 1.143-.011 2.061-.011 2.35 0 .224.156.491.573.405a8.365 8.365 0 0 0 4.11-3.116 8.707 8.707 0 0 0 1.567-4.99c0-4.721-3.73-8.545-8.334-8.545Z"
      />
    </svg>
  );
}

export const Stargazer = ({ count }: { count: number }) => {
  return (
    <div className="flex">
      <Link
        href="https://github.com/tryanything-ai/anything"
        className="caption-s flex items-center gap-1 rounded-l-[4px] border border-slate-300 bg-slate-100 px-2 py-[2px] font-medium text-slate-800 hover:border-slate-400 hover:bg-slate-200"
      >
        <GitHubIcon className="h-5 w-5 fill-slate-800" />
        <span className="">Star</span>
      </Link>
      <Link
        href="https://github.com/tryanything-ai/anything"
        className="group inline-flex items-center rounded-r-[4px] border-y border-r border-slate-300 bg-white px-2 py-[2px] hover:bg-slate-50"
      >
        <span className="caption-s font-medium text-slate-800 group-hover:text-crimson-600">
          {count}
        </span>
      </Link>
    </div>
  );
};
