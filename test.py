# This file is part of pycavalry.
#
# pycavalry is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published
# by the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

from typing import reveal_type


(lambda x, y, z: "asdf")(1, 2, 3)

a = lambda x, y, z: "asdf"
reveal_type(a(1, 2, 3))


def do(a: int, b: float):
    return b
    return a


reveal_type(do)


class Test:
    pass


reveal_type(Test)
