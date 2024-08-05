//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef USER_PROFILE_ADAPTER_H
#define USER_PROFILE_ADAPTER_H

#include <nlohmann/json.hpp>
#include <session/config/user_profile.hpp>

namespace session::config
{
    void to_json(nlohmann::json &, const UserProfile &);
}

#endif // USER_PROFILE_ADAPTER_H
