//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef CONTACTS_ADAPTER_H
#define CONTACTS_ADAPTER_H

#include <nlohmann/json.hpp>
#include <session/config/contacts.hpp>

namespace session::config
{
    void to_json(nlohmann::json &out, const Contacts &c);
}

#endif // CONTACTS_ADAPTER_H
