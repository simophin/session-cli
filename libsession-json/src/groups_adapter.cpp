//
// Created by Fanchao Liu on 28/6/2024.
//

#include "groups_adapter.h"

#include "common_adapters.h"

using namespace nlohmann;

namespace session::config::groups
{
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(
        member,
        session_id, name, profile_picture, admin, supplement, invite_status, promotion_status,
        removed_status);

    void to_json(json &out, const Info &c)
    {
        out["id"] = c.id;
        out["name"] = c.get_name();
        out["description"] = c.get_description();
        out["profile_pic"] = c.get_profile_pic();
        out["expiry_timer"] = c.get_expiry_timer();
        out["created"] = c.get_created();
        out["delete_before"] = c.get_delete_before();
        out["delete_attach_before"] = c.get_delete_attach_before();
    }

    void to_json(json &out, const Members &members)
    {
        out = json::array();
        for (auto iter = members.begin(); iter != members.end(); ++iter)
        {
            out.push_back(*iter);
        }
    }
}
