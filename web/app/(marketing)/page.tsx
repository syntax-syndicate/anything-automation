"use client";
import { useEffect, useState } from "react";
import Link from "next/link";

// import { BgPattern } from "@/components/ui/Bgpattern";
import { SignUpButton } from "@/components/marketing/LandingSignUp";
import { Button } from "@/components/ui/Button";

export default function IndexPage() {
  const [referrerDomain, setReferrerDomain] = useState<string>("");
  // Effect to run when the component mounts
  useEffect(() => {
    // Get the referrer URL
    const referrerUrl = document.referrer; // Extract the domain from the URL
    if (referrerUrl) {
      const url = new URL(referrerUrl);
      const domain = url.hostname;
      // Update the state variable
      setReferrerDomain(domain);
    }
  }, []);

  return (
    <>
      {/* Bg Pattern */}
      {/* <BgPattern /> */}
      {/* Hero Copy */}
      <div className="mt-16 flex flex-col items-center gap-4 text-5xl">
        <h1 className="md:display h2 w-full px-4 text-center md:w-[900px] md:px-0">
          The simplest way to build{" "}
          <span className="text-crimson-9">AI Automations</span> for your
          startup
        </h1>
        {/* <p className="body-xl px-4 text-center text-slate-11 md:w-[572px] md:px-0">
          Put an end to your creative block, get help from your AI creative
          writer
        </p> */}
      </div>
      {/* Hero CTA */}
      <div className="mb-40 mt-20 flex flex-col items-center gap-4">
        {/* <p className="body">
          Get your <span className="font-semibold">free account today</span>
        </p> */}
        <div className="flex flex-col items-center gap-2 md:flex-row md:gap-4">
          {/* <SignUpButton className="block" />
           */}
          <Button
            variant={"primary"}
            href={`https://airtable.com/shrfQYBtcoUqYNylu?prefill_fldVLaD0gtTpY1jxP=wysiwyg&hide_fldVLaD0gtTpY1jxP=true&prefill_referring_domain=${referrerDomain}&hide_referring_domain=true`}
            target="_blank"
            rel="noopener noreferrer"
          >
            Get Early Access
          </Button>
        </div>
        {/* <p className="caption text-slate-11">No credit card required</p> */}
      </div>
    </>
  );
}
