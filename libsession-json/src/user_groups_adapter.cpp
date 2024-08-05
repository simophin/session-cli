//
// Created by Fanchao Liu on 28/6/2024.
//

#include "user_groups_adapter.h"
#include "common_adapters.h"

using namespace nlohmann;

namespace session::config
{
    // Group info
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(base_group_info, priority, name, invited, joined_at,
                                                      mute_until, notifications);

    void to_json(json &j, const legacy_group_info &info)
    {
        to_json(j, static_cast<const base_group_info &>(info));
        json members;
        for (const auto &[session_id, admin] : info.members())
        {
            json member;
            member["session_id"] = session_id;
            member["admin"] = admin;
            members += member;
        }
        j["session_id"] = info.session_id;
        j["enc_pubkey"] = Base64{info.enc_pubkey};
        j["enc_seckey"] = Base64{info.enc_seckey};
        j["disappearing_timer"] = info.disappearing_timer;
        j["members"] = members;
    }

    void to_json(json &j, const group_info &info)
    {
        to_json(j, static_cast<const base_group_info &>(info));

        j["id"] = info.id;
        j["secret_key"] = Base64{info.secretkey};
        j["auth_data"] = Base64{info.auth_data};
    }

    void to_json(json &j, const community_info &info)
    {
        to_json(j, static_cast<const base_group_info &>(info));

        j["url"] = info.base_url() + "/" + info.room_norm();
        j["pub_key"] = info.pubkey_hex();
    }

    void to_json(json &j, const any_group_info &info)
    {
        const char *group_type;

        if (const auto group = std::get_if<group_info>(&info))
        {
            j = *group;
            group_type = "group";
        }
        else if (const auto legacy_group = std::get_if<legacy_group_info>(&info))
        {
            j = *legacy_group;
            group_type = "legacy_group";
        }
        else if (const auto community_info = std::get_if<config::community_info>(&info))
        {
            j = *community_info;
            group_type = "community";
        }
        else
        {
            return;
        }

        j["type"] = group_type;
    }

    void to_json(json &out, const UserGroups &user_groups)
    {
        out = json::array();
        for (auto iter = user_groups.begin(); iter != user_groups.end(); ++iter)
        {
            out.push_back(*iter);
        }
    }
}
