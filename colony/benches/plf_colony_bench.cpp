#include "plf_colony.h"
#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <vector>

namespace {

using value_type = std::uintptr_t;
using colony_type = plf::colony<value_type>;
constexpr std::uint64_t sparse_load_denominator = 100;

struct PlfColony {
    colony_type colony;
    std::vector<colony_type::iterator> handles;
};

struct Rng {
    std::uint64_t state;

    explicit Rng(std::uint64_t seed) : state(seed) {}

    std::uint64_t next() {
        state += 0x9e3779b97f4a7c15ULL;
        std::uint64_t z = state;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        return z ^ (z >> 31);
    }

    std::size_t index(std::size_t upper) {
        return static_cast<std::size_t>(next() % upper);
    }
};

bool sparse_keep(std::size_t index, std::size_t size) {
    std::uint64_t value = static_cast<std::uint64_t>(index) ^
        (static_cast<std::uint64_t>(size) * 0x9e3779b97f4a7c15ULL);
    value += 0x9e3779b97f4a7c15ULL;
    value = (value ^ (value >> 30)) * 0xbf58476d1ce4e5b9ULL;
    value = (value ^ (value >> 27)) * 0x94d049bb133111ebULL;
    value = value ^ (value >> 31);
    return (value % sparse_load_denominator) == 0;
}

PlfColony* fill_colony(std::size_t size) {
    auto* container = new PlfColony();
    container->colony.reserve(size);
    container->handles.reserve(size);
    for (std::size_t value = 0; value != size; ++value) {
        container->handles.push_back(container->colony.insert(value));
    }
    return container;
}

PlfColony* make_sparse_colony(std::size_t size) {
    auto* container = fill_colony(size);
    std::vector<colony_type::iterator> survivors;
    survivors.reserve((size + sparse_load_denominator - 1) / sparse_load_denominator);

    for (std::size_t index = 0; index != container->handles.size(); ++index) {
        auto handle = container->handles[index];
        if (sparse_keep(index, size)) {
            survivors.push_back(handle);
        } else {
            container->colony.erase(handle);
        }
    }

    container->handles = std::move(survivors);
    return container;
}

} // namespace

extern "C" {

PlfColony* plf_colony_fill(std::size_t size) {
    return fill_colony(size);
}

PlfColony* plf_colony_sparse(std::size_t size) {
    return make_sparse_colony(size);
}

void plf_colony_free(PlfColony* container) {
    delete container;
}

std::size_t plf_colony_len(const PlfColony* container) {
    return container->colony.size();
}

std::size_t plf_colony_insert_loop(std::size_t size) {
    colony_type colony;
    colony.reserve(size);
    std::size_t sum = 0;

    for (std::size_t value = 0; value != size; ++value) {
        auto it = colony.insert(value);
        sum += *it;
    }

    return sum + colony.size();
}

std::size_t plf_colony_iter_loop(const PlfColony* container) {
    std::size_t sum = 0;
    for (const auto& value : container->colony) {
        sum += value;
    }
    return sum;
}

std::size_t plf_colony_get_random_loop(const PlfColony* container, const std::size_t* indices, std::size_t len) {
    std::size_t sum = 0;
    for (std::size_t i = 0; i != len; ++i) {
        sum += *container->handles[indices[i]];
    }
    return sum;
}

std::size_t plf_colony_random_churn_loop(std::size_t size, std::size_t ops, std::uint64_t seed) {
    PlfColony* container = fill_colony(size);
    Rng rng(seed);
    std::size_t next_value = size;
    std::size_t sum = 0;

    for (std::size_t op = 0; op != ops; ++op) {
        const std::size_t action = rng.index(100);
        if (action < 45) {
            const std::size_t index = rng.index(container->handles.size());
            sum += *container->handles[index];
        } else {
            const std::size_t index = rng.index(container->handles.size());
            auto handle = container->handles[index];
            sum += *handle;
            container->colony.erase(handle);
            container->handles[index] = container->handles.back();
            container->handles.pop_back();
            container->handles.push_back(container->colony.insert(next_value++));
        }
    }

    sum += container->colony.size();
    delete container;
    return sum;
}

std::size_t plf_colony_churn_sparse_loop(std::size_t size, std::size_t ops, std::uint64_t seed) {
    PlfColony* container = make_sparse_colony(size);
    Rng rng(seed);
    std::size_t next_value = size;
    std::size_t sum = 0;

    for (std::size_t op = 0; op != ops; ++op) {
        const std::size_t index = rng.index(container->handles.size());
        auto handle = container->handles[index];
        sum += *handle;
        container->colony.erase(handle);
        container->handles[index] = container->handles.back();
        container->handles.pop_back();
        container->handles.push_back(container->colony.insert(next_value++));
    }

    sum += container->colony.size();
    delete container;
    return sum;
}

} // extern "C"
