//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef CONVO_INFO_ADAPTERS_H
#define CONVO_INFO_ADAPTERS_H

#include <nlohmann/json.hpp>
#include <session/config/convo_info_volatile.hpp>

namespace session::config
{
    void to_json(nlohmann::json &out, const ConvoInfoVolatile &c);
}

#endif // CONVO_INFO_ADAPTERS_H
