import { ProfileLinks } from "@repo/ui/components/profileLinks";
import { TemplateGrid } from "@repo/ui/components/templateGrid";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";

import { Avatar } from "@/components/avatar";

// export const generateStaticParams = async () => {
//   let profiles = await fetchProfiles();
//   // has username key to populate route
//   console.log("profiles in generateStaticParams", profiles);
//   if (!profiles) return [];
//   let goodProfiles = profiles
//     .filter((profile: any) => profile.username !== null)
//     .map((profile: any) => profile.username);
//   return goodProfiles;
// };

export default async function Profile({
  params,
}: {
  params: { username: string };
}) {
  //weird hack problem with base og image firing this route with params
  //{ username: 'opengraph-image' }
  if (params.username === "opengraph-image") {
    notFound();
  }

  // console.log("params in ProfilePage", params);
  // const profile = await fetchProfile(params.username);
  // const templates = await fetchProfileTemplates(params.username);
  //TODO: ungut this
  const profile = false;
  const templates = false;
  if (!profile || !templates) {
    //only show users that exist with templates
    notFound();
  }

  return (
    <div className="mx-auto my-6 flex flex-col md:my-16 md:flex-row">
      {/* Left Column */}
      <div className="h-full max-w-sm p-6">
        <div className="avatar">
          <div className="w-24 rounded-full">
            {/* <Image
              width={100}
              height={100}
              src={profile.avatar_url ? profile.avatar_url : ""}
              alt={profile.username ? profile.username : "user profile picture"}
            /> */}
          </div>
        </div>
        {/* <div className="text-3xl">{profile.full_name}</div>
        <div className="mt-2 opacity-70">@{profile.username}</div>
        <div className="mt-2">{profile.bio}</div> */}
        {/* <ProfileLinks profile={profile} Link={Link as any} /> */}
      </div>
      {/* Right Column */}
      <div className="flex flex-col p-2 md:pl-5">
        <div className="pb-4 pl-2 text-2xl">Templates</div>
        <div className="items-center">
          <TemplateGrid
            LinkComponent={Link as any}
            AvatarComponent={Avatar}
            templates={templates}
            profile={false}
          />
        </div>
      </div>
    </div>
  );
}
