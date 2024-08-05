//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef GROUPS_ADAPTER_H
#define GROUPS_ADAPTER_H

#include <nlohmann/json.hpp>

#include <session/config/groups/info.hpp>
#include <session/config/groups/members.hpp>

namespace session::config::groups
{
    void to_json(nlohmann::json &out, const Info &c);

    void to_json(nlohmann::json &out, const Members &members);
}

#endif // GROUPS_ADAPTER_H
