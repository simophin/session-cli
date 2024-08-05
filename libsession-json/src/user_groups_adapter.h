//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef USER_GROUPS_ADAPTER_H
#define USER_GROUPS_ADAPTER_H

#include <nlohmann/json.hpp>
#include <session/config/user_groups.hpp>

namespace session::config
{
    void to_json(nlohmann::json &out, const UserGroups &c);
}

#endif // USER_GROUPS_ADAPTER_H
