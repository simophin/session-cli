//
// Created by Fanchao Liu on 28/6/2024.
//

#include "user_profile_adapter.h"
#include "common_adapters.h"

namespace session::config
{
    void to_json(nlohmann::json &out, const UserProfile &c)
    {
        out["name"] = c.get_name();
        out["profile_pic"] = c.get_profile_pic();
        out["blinded_msgreqs"] = c.get_blinded_msgreqs();
        out["nts_priority"] = c.get_nts_priority();
        out["nts_expiry"] = c.get_nts_expiry();
    }
}
