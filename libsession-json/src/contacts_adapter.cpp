//
// Created by Fanchao Liu on 28/6/2024.
//

#include "contacts_adapter.h"

#include "common_adapters.h"

namespace session::config
{
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(
        contact_info,
        session_id, name, nickname, profile_picture, approved,
        approved_me, blocked, priority, notifications, mute_until,
        exp_mode, exp_timer, created);

    void to_json(nlohmann::json &out, const Contacts &c)
    {
        out = nlohmann::json::array();
        for (auto iter = c.begin(); iter != c.end(); ++iter)
        {
            out.push_back(*iter);
        }
    }
}
